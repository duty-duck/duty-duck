use anyhow::Context;
use chrono::{DateTime, Utc};
use opentelemetry_proto::tonic::{
    common::v1::{any_value, AnyValue, ArrayValue, KeyValue, KeyValueList},
    logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
    resource::v1::Resource,
};
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;
use utoipa::ToSchema;

use crate::domain::{
    entities::{
        authorization::{AuthContext, Permission},
        task::{
            get_task_aggregate, save_task_aggregate, RunningTaskAggregate, TaskAggregate, TaskId,
        },
    },
    ports::{
        logs_ingestor::LogsIngestor, task_repository::TaskRepository,
        task_run_repository::TaskRunRepository,
    },
};

#[derive(Error, Debug)]
pub enum SendTaskLogsError {
    #[error("Task not found")]
    TaskNotFound,
    #[error("Task is not running")]
    TaskIsNotRunning,
    #[error("User is not allowed to send a heartbeat for this task")]
    Forbidden,
    #[error("Technical error")]
    TechnicalFailure(#[from] anyhow::Error),
}

/// A request to store the logs associated with a running task so they can be consulted and searched.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SendTaskLogsRequest {
    events: Vec<TaskLogEvent>,
}

/// A single log event from a task run. (By default, each new line in the standard output is considered a seperate event)
#[derive(Debug, Deserialize, ToSchema)]
pub struct TaskLogEvent {
    /// severity text (also known as log level). This is the original string representation of the severity as it is known at the source, for instance "DEBUG", or "ERROR".
    /// This default behaviour of the DutyDuck CLI is to treat the standard output of the process as "INFO" and the standard error as "ERROR".
    severity_text: Option<String>,
    /// SeverityNumber is an integer number. Smaller numerical values correspond to less severe events (such as debug events), larger numerical values correspond to more severe events (such as errors and critical events).
    /// The meaning of this value is defined by the OpenTelemetry standard.
    /// This default behaviour of the DutyDuck CLI is to treat the standard output of the process as "INFO" (severity number = 9) and the standard error as "ERROR" (severity_number = 17).
    severity_number: Option<i32>,
    /// The body of the log. This can be unstructured data (i.e. a string of text) or a structured map of keys and values
    /// If structured data is desired, the client is responsible for supplying the structured data.
    /// The default behaviour of the DutyDuck CLI is to capture the raw standard output and standard error streams of the process as unstructured data.
    body: Value,
    /// The timestamp at which the log occured, as measured by the orgin clock
    timestamp: DateTime<Utc>,
}

pub async fn send_task_logs_use_case<TR, TRR, LI>(
    auth_context: &AuthContext,
    task_repository: &TR,
    task_run_repository: &TRR,
    logs_ingestor: &LI,
    task_id: TaskId,
    request: SendTaskLogsRequest,
) -> Result<(), SendTaskLogsError>
where
    TR: TaskRepository,
    TRR: TaskRunRepository<Transaction = TR::Transaction>,
    LI: LogsIngestor,
{
    if !auth_context.can(Permission::WriteTaskRuns) {
        return Err(SendTaskLogsError::Forbidden);
    }

    let mut tx = task_repository.begin_transaction().await?;
    let aggregate = get_task_aggregate(
        task_repository,
        task_run_repository,
        &mut tx,
        auth_context.active_organization_id,
        &task_id,
    )
    .await?;
    let now = Utc::now();

    let running_aggregate: RunningTaskAggregate = match aggregate {
        None => return Err(SendTaskLogsError::TaskNotFound),
        Some(TaskAggregate::Running(t)) =>
        // sending a log also counts as sending a hearbeat (it's a signal that the task is alive and well)
        {
            t.receive_heartbeat(now)
                .context("failed to receive heartbeat")?
        }
        Some(_) => return Err(SendTaskLogsError::TaskIsNotRunning),
    };

    let resource = task_aggregate_to_resource(&running_aggregate);

    save_task_aggregate(
        task_repository,
        task_run_repository,
        &mut tx,
        TaskAggregate::Running(running_aggregate),
    )
    .await?;

    // commit the transaction before ingesting the log to avoid keeping the transaction open during the ingestion
    task_repository.commit_transaction(tx).await?;

    logs_ingestor
        .ingest_logs(vec![ResourceLogs {
            schema_url: String::new(),
            resource: Some(resource),
            scope_logs: vec![ScopeLogs {
                scope: None,
                schema_url: String::new(),
                log_records: request
                    .events
                    .into_iter()
                    .map(log_event_to_log_record)
                    .collect(),
            }],
        }])
        .await?;

    Ok(())
}

fn task_aggregate_to_resource(aggregate: &RunningTaskAggregate) -> Resource {
    let task_id = *aggregate.task().base().id();
    let task_run_id = *aggregate.task_run().id();

    let attributes = vec![
        KeyValue {
            key: "service.namespace".to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue("dutyduck.Task".to_string())),
            }),
        },
        KeyValue {
            key: "service.name".to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(task_id.to_string())),
            }),
        },
        KeyValue {
            key: "service.instance.id".to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(task_run_id.to_string())),
            }),
        },
    ];
    Resource {
        attributes,
        dropped_attributes_count: 0,
    }
}

fn log_event_to_log_record(event: TaskLogEvent) -> LogRecord {
    // note: despite the name of the field, the back-end supports timestamps expressed as seconds, milliseconds or microseconds. The precision is inferred from the number of digits
    let timestamp = event.timestamp.timestamp_millis() as u64;

    LogRecord {
        time_unix_nano: timestamp,
        observed_time_unix_nano: timestamp,
        severity_number: event.severity_number.unwrap_or_default(),
        severity_text: event.severity_text.unwrap_or_default(),
        body: Some(json_to_any_value(event.body)),
        attributes: vec![],
        dropped_attributes_count: 0,
        flags: 0,
        trace_id: vec![],
        span_id: vec![],
        event_name: String::new(),
    }
}

fn json_to_any_value(value: Value) -> AnyValue {
    match value {
        Value::Bool(b) => AnyValue {
            value: Some(any_value::Value::BoolValue(b)),
        },
        Value::String(str) => AnyValue {
            value: Some(any_value::Value::StringValue(str)),
        },
        Value::Object(map) => AnyValue {
            value: Some(any_value::Value::KvlistValue(KeyValueList {
                values: map
                    .into_iter()
                    .map(|(key, v)| KeyValue {
                        key,
                        value: Some(json_to_any_value(v)),
                    })
                    .collect(),
            })),
        },
        Value::Null => AnyValue { value: None },
        Value::Number(n) => {
            let value = n
                .as_f64()
                .map(any_value::Value::DoubleValue)
                .or_else(|| n.as_i64().map(any_value::Value::IntValue));

            AnyValue { value }
        }
        Value::Array(arr) => AnyValue {
            value: Some(any_value::Value::ArrayValue(ArrayValue {
                values: arr.into_iter().map(json_to_any_value).collect(),
            })),
        },
    }
}
