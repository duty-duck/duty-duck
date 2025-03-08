use chrono::{DateTime, Utc};
use opentelemetry::{
    json::OTELLogDocument,
    proto::{
        common::v1::{any_value, AnyValue, KeyValue},
        json_to_anyvalue,
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
        resource::v1::Resource,
    },
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::domain::entities::task::RunningTaskAggregate;

/// A type to hold a TaskRun aggregate and its related logs at the same time.
/// This type has a conversion into Open Telemetry's [ResourceLogs]
pub struct TaskRunLogEvents<'aggregate> {
    pub task_aggregate: &'aggregate RunningTaskAggregate,
    pub log_events: Vec<TaskRunLogEvent>,
}

/// A single log event from a task run. (By default, each new line in the standard output is considered a seperate event)
/// This type has a conversion from and to the OpenTelemetry format, which we can use to ingest task run logs and retrieve them back later
#[derive(Debug, Deserialize, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct TaskRunLogEvent {
    /// severity text (also known as log level). This is the original string representation of the severity as it is known at the source, for instance "DEBUG", or "ERROR".
    /// This default behaviour of the DutyDuck CLI is to treat the standard output of the process as "INFO" and the standard error as "ERROR".
    pub severity_text: Option<String>,
    /// SeverityNumber is an integer number. Smaller numerical values correspond to less severe events (such as debug events), larger numerical values correspond to more severe events (such as errors and critical events).
    /// The meaning of this value is defined by the OpenTelemetry standard.
    /// This default behaviour of the DutyDuck CLI is to treat the standard output of the process as "INFO" (severity number = 9) and the standard error as "ERROR" (severity_number = 17).
    pub severity_number: Option<i32>,
    /// The body of the log. This can be unstructured data (i.e. a string of text) or a structured map of keys and values
    /// If structured data is desired, the client is responsible for supplying the structured data.
    /// The default behaviour of the DutyDuck CLI is to capture the raw standard output and standard error streams of the process as unstructured data.
    #[ts(type = "any")]
    pub body: serde_json::Value,
    /// The timestamp at which the log occured, as measured by the orgin clock
    pub timestamp: DateTime<Utc>,
}

impl<'a> From<TaskRunLogEvents<'a>> for ResourceLogs {
    fn from(
        TaskRunLogEvents {
            task_aggregate,
            log_events,
        }: TaskRunLogEvents<'a>,
    ) -> Self {
        let task_name = task_aggregate.task().base().user_id();
        let task_run_id = *task_aggregate.task_run().id();

        let resource_attributes = vec![
            KeyValue {
                key: "service.namespace".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue("dutyduck.Task".to_string())),
                }),
            },
            KeyValue {
                key: "service.name".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue(task_name.to_string())),
                }),
            },
            KeyValue {
                key: "service.instance.id".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue(task_run_id.to_string())),
                }),
            },
        ];
        let resource = Resource {
            attributes: resource_attributes,
            dropped_attributes_count: 0,
        };

        ResourceLogs {
            schema_url: String::new(),
            resource: Some(resource),
            scope_logs: vec![ScopeLogs {
                scope: None,
                schema_url: String::new(),
                log_records: log_events.into_iter().map(LogRecord::from).collect(),
            }],
        }
    }
}

/// Conversion of [TaskRunLogEvent] to Opentelemetry's [LogRecord] (protobuf format)
/// This conversion is used when ingesting task run logs
impl From<TaskRunLogEvent> for LogRecord {
    fn from(event: TaskRunLogEvent) -> Self {
        // note: despite the name of the field, the back-end supports timestamps expressed as seconds, milliseconds or microseconds. The precision is inferred from the number of digits
        let timestamp = event.timestamp.timestamp_millis() as u64;

        LogRecord {
            time_unix_nano: timestamp,
            observed_time_unix_nano: timestamp,
            severity_number: event.severity_number.unwrap_or_default(),
            severity_text: event.severity_text.unwrap_or_default(),
            body: Some(json_to_anyvalue(event.body)),
            attributes: vec![],
            dropped_attributes_count: 0,
            flags: 0,
            trace_id: vec![],
            span_id: vec![],
            event_name: String::new(),
        }
    }
}

/// Conversion from the [OTELLogDocument] JSON document, returned by the searcher, to a [TaskRunLogEvent] which is served by the task runs API
/// This conversion is used when searching task run logs
impl From<OTELLogDocument> for TaskRunLogEvent {
    fn from(value: OTELLogDocument) -> Self {
        TaskRunLogEvent {
            severity_text: value.severity_text,
            severity_number: value.severity_number,
            body: value
                .body
                .map(serde_json::Value::Object)
                .unwrap_or_default(),
            timestamp: DateTime::from_timestamp_nanos(value.observed_timestamp_unix),
        }
    }
}
