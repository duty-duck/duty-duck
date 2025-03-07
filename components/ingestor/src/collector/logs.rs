use std::sync::Arc;

use moka::future::Cache;
use opentelemetry_proto::tonic::{
    collector::logs::v1::{
        logs_service_server::LogsService, ExportLogsServiceRequest, ExportLogsServiceResponse,
    },
    common::v1::{any_value, AnyValue, KeyValue},
    logs::v1::ResourceLogs,
};
use quickwit_client_rs::indexes_api_v1::{
    otel::{otel_logs_doc_mapping, OTELInstrumentationScope, OTELLogInput, OTELResource},
    IndexConfig, IndexingResources, IndexingSettings, RetentionSettings, SearchSettings,
};
use serde_json::{Number, Value};
use tonic::{Request, Response, Status};
use tracing::error;
use uuid::Uuid;

use crate::{
    auth::{AuthenticatedToken, Authenticator},
    quickwit::QuickwitClusterClient,
};

/// Implements the OpenTelemetry [LogsService] by forwarding logs to Quickwit's indexers (after authenticating the requests)
#[derive(Clone)]
pub struct LogsIngestorService {
    authenticator: Authenticator,
    quickwit_clients: Vec<QuickwitClusterClient>,
    /// a cache to store the fact that the logs index for a given organization exists, and obtain its name
    /// if an entry exists in this cache for a given organization, we can route ingest requests directly to the index without querying the indexes API first to ensure it exists
    /// if not, we use the indexes API to test whether the index exists, and create it on the fly if it does not
    logs_index_v1_cache: Arc<Cache<Uuid, String>>,
}

#[tonic::async_trait]
impl LogsService for LogsIngestorService {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> tonic::Result<Response<ExportLogsServiceResponse>> {
        let auth_context = self
            .authenticator
            .authenticate_tonic_request(&request)
            .await?;

        let index_id = self
            .ensure_logs_index_v1_exists(auth_context.org_id)
            .await.map_err(|error| {
                error!(?error, "Something went wrong while ensuring the destination index exists for one or more log events");
                Status::internal("Internal error while determining destination index. Please try again later")
            })?;

        let request = request.into_inner();

        for (logs, resource) in request.resource_logs.into_iter().map(convert_otel_log) {
            let partition_key = resource_partition_key(resource.as_ref(), &auth_context);
            let client = self
                .quickwit_client(auth_context.org_id)
                .get_client(&partition_key)
                .ingest_api_v1();
            client
                .ingest_documents(
                    &index_id,
                    logs,
                    quickwit_client_rs::ingest_api_v1::Commit::Auto,
                )
                .await
                .map_err(|error| {
                    error!(?error, "Something went wrong while indexing logs");
                    Status::internal("Internal error while indexing. Please try again later")
                })?;
        }

        let response = Response::new(ExportLogsServiceResponse {
            partial_success: None,
        });

        Ok(response)
    }
}

impl LogsIngestorService {
    pub fn new(
        quickwit_clients: Vec<QuickwitClusterClient>,
        authenticator: Authenticator,
    ) -> anyhow::Result<Self> {
        if quickwit_clients.is_empty() {
            anyhow::bail!("cannot create logs ingestor with 0 quickwit clusters");
        }

        Ok(Self {
            quickwit_clients,
            authenticator,
            logs_index_v1_cache: Arc::new(Cache::new(1000)),
        })
    }

    fn quickwit_client(&self, organization_id: Uuid) -> &QuickwitClusterClient {
        &self.quickwit_clients
            [crc32fast::hash(organization_id.as_bytes()) as usize % self.quickwit_clients.len()]
    }

    /// Ensures the log index V1 exists for a given organization and returns its name
    async fn ensure_logs_index_v1_exists(&self, organization_id: Uuid) -> anyhow::Result<String> {
        if let Some(index_name) = self.logs_index_v1_cache.get(&organization_id) {
            return Ok(index_name);
        }

        // compute the name of the desintation index from the id of the organization
        // we isolate logs from multiple tenants using the organisation id, and add a prefix with a version number to leave open the possibility of future developments
        let destination_index_name = format!("{}-logs-index-v1", organization_id);
        let client = self
            .quickwit_client(organization_id)
            .get_any_client()
            .indexes_api_v1();

        if client
            .describe_index(&destination_index_name)
            .await?
            .is_none()
        {
            let index_config = logs_index_v1_config(&destination_index_name);
            client.create_index(&index_config).await?;
        }
        self.logs_index_v1_cache
            .insert(organization_id, destination_index_name.clone())
            .await;
        Ok(destination_index_name)
    }
}

fn convert_otel_log(log: ResourceLogs) -> (Vec<OTELLogInput>, Option<OTELResource>) {
    let resource = log.resource.map(|r| OTELResource {
        attributes: convert_attributes(r.attributes),
        dropped_attributes_count: r.dropped_attributes_count,
    });

    let logs = log
        .scope_logs
        .into_iter()
        .flat_map(|logs| {
            let scope = logs.scope.map(|s| OTELInstrumentationScope {
                name: s.name,
                version: s.version,
                attributes: convert_attributes(s.attributes),
                dropped_attributes_count: s.dropped_attributes_count,
            });

            logs.log_records
                .into_iter()
                .map(|record| OTELLogInput {
                    resource: resource.clone(),
                    scope: scope.clone(),
                    timestamp_unix: Some(record.time_unix_nano as i64).filter(|ts| *ts > 0),
                    observed_timestamp_unix: record.observed_time_unix_nano as i64,
                    severity_text: Some(record.severity_text).filter(|t| !t.is_empty()),
                    severity_number: Some(record.severity_number).filter(|s| *s > 0),
                    body: record.body.and_then(convert_any_value),
                    trace_id_hex: Some(record.trace_id)
                        .filter(|t| !t.is_empty())
                        .map(hex::encode),
                    span_id_hex: Some(record.span_id)
                        .filter(|t| !t.is_empty())
                        .map(hex::encode),
                    trace_flags: Some(record.flags),
                    attributes: Some(convert_attributes(record.attributes)),
                    dropped_attributes_count: record.dropped_attributes_count,
                })
                .collect::<Vec<_>>()
        })
        .collect();

    (logs, resource)
}

/// Determines the partition key (used to select the quickwit client) from a given OTEL resource and authentication context
fn resource_partition_key(
    resource: Option<&OTELResource>,
    auth_context: &AuthenticatedToken,
) -> String {
    let org_partition_key = auth_context.org_id.to_string();
    let resource_partition_key = resource.map(|r| &r.attributes).and_then(|attrs| {
        // use the unique id of the service to partition requests
        attrs
            .get("service.instance.id")
            // or else the name of the service
            .or_else(|| attrs.get("service.name"))
            // or else the unique id of the compute unit (see https://opentelemetry.io/docs/specs/semconv/resource/#compute-unit)
            .or_else(|| attrs.get("container.id"))
            .or_else(|| attrs.get("cloud.resource_id"))
            // or else the unique id of the host
            .or_else(|| attrs.get("host.id"))
    });

    match (org_partition_key, resource_partition_key) {
        (org, Some(Value::String(resource))) => format!("{org}:{resource}"),
        (org, Some(Value::Number(resource))) => format!("{org}:{resource}"),
        (org, _) => org,
    }
}

fn convert_attributes(attributes: Vec<KeyValue>) -> serde_json::Map<String, Value> {
    attributes
        .into_iter()
        .filter_map(|kv| Some((kv.key, kv.value.and_then(convert_any_value)?)))
        .collect()
}

fn convert_any_value(value: AnyValue) -> Option<Value> {
    match value.value {
        Some(any_value::Value::StringValue(v)) => Some(Value::String(v)),
        Some(any_value::Value::BoolValue(v)) => Some(Value::Bool(v)),
        Some(any_value::Value::IntValue(v)) => Some(Value::Number(Number::from(v))),
        Some(any_value::Value::DoubleValue(v)) => Some(Value::Number(Number::from_f64(v)?)),
        Some(any_value::Value::ArrayValue(arr)) => Some(Value::Array(
            arr.values
                .into_iter()
                .filter_map(convert_any_value)
                .collect(),
        )),
        Some(any_value::Value::KvlistValue(kv_list)) => {
            Some(Value::Object(convert_attributes(kv_list.values)))
        }
        // We do not support indexing bytes at the moment
        Some(any_value::Value::BytesValue(_)) => None,
        None => None,
    }
}

fn logs_index_v1_config(index_name: &str) -> IndexConfig {
    IndexConfig {
        index_id: index_name.to_string(),
        version: quickwit_client_rs::indexes_api_v1::Version::V08,
        doc_mapping: otel_logs_doc_mapping(),
        retention: Some(RetentionSettings {
            period: "2 months".to_string(),
            schedule: "daily".to_string(),
        }),
        search_settings: Some(SearchSettings {
            default_search_fields: Some(vec!["body.message".to_string()]),
        }),
        indexing_settings: Some(IndexingSettings {
            commit_timeout_secs: Some(30),
            docstore_blocksize: None,
            resources: Some(IndexingResources {
                heap_size: Some("256 MB".to_string()),
            }),
        }),
    }
}
