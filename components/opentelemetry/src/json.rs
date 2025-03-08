use opentelemetry_proto::tonic::{
    common::v1::{any_value, AnyValue, InstrumentationScope, KeyValue},
    logs::v1::LogRecord,
    resource::v1::Resource,
};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OTELResource {
    pub attributes: serde_json::Map<String, Value>,
    pub dropped_attributes_count: u32,
}

impl From<Resource> for OTELResource {
    fn from(r: Resource) -> Self {
        OTELResource {
            attributes: otel_attributes_to_json(r.attributes),
            dropped_attributes_count: r.dropped_attributes_count,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OTELInstrumentationScope {
    pub name: String,
    pub version: String,
    pub attributes: serde_json::Map<String, Value>,
    pub dropped_attributes_count: u32,
}

impl From<InstrumentationScope> for OTELInstrumentationScope {
    fn from(s: InstrumentationScope) -> Self {
        OTELInstrumentationScope {
            name: s.name,
            version: s.version,
            attributes: otel_attributes_to_json(s.attributes),
            dropped_attributes_count: s.dropped_attributes_count,
        }
    }
}

serde_with::with_prefix!(prefix_resource "resource_");
serde_with::with_prefix!(prefix_scope "scope_");

/// Represents a JSON Document for a single line of log, that matches the Open Telemetry Standard and doc mapping of the Quickwit index.
///
/// Quickwit can index any JSON document (with or without a srict schema), but this particular dcument is meant to be converted back and forth
/// between the JSON that is indexed in Quickwit and the protobuf representaion in Opentelemetry.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct OTELLogDocument {
    /// Time when the event occurred measured by the origin clock, i.e. the time at the source.
    /// This field is optional, it may be missing if the source timestamp is unknown
    /// The precision is inferred from the number of digits
    #[serde(rename = "timestamp_nanos")]
    pub timestamp_unix: Option<i64>,

    /// Time when the event was observed by the collection system. For events that originate in OpenTelemetry (e.g. using OpenTelemetry Logging SDK) this timestamp is typically set at the generation time and is equal to Timestamp.
    /// For events originating externally and collected by OpenTelemetry (e.g. using Collector) this is the time when OpenTelemetry’s code observed the event measured by the clock of the OpenTelemetry code.
    /// This field SHOULD be set once the event is observed by OpenTelemetry.
    #[serde(rename = "observed_timestamp_nanos")]
    pub observed_timestamp_unix: i64,

    /// severity text (also known as log level). This is the original string representation of the severity as it is known at the source.
    /// If this field is missing and SeverityNumber is present then the short name that corresponds to the SeverityNumber may be used as a substitution.
    /// This field is optional
    pub severity_text: Option<String>,

    /// numerical value of the severity, normalized to values described in this document. This field is optional.
    /// SeverityNumber is an integer number. Smaller numerical values correspond to less severe events (such as debug events), larger numerical values correspond to more severe events (such as errors and critical events).
    pub severity_number: Option<i32>,

    /// A value containing the body of the log record. Can be for example a human-readable string message (including multi-line) describing the event in a free form
    /// or it can be a structured data composed of arrays and maps of other values
    pub body: Option<serde_json::Map<String, Value>>,

    /// Describes the source of the log, aka resource. Multiple occurrences of events coming from the same event source can happen across time and they all have the same value of Resource.
    /// Can contain for example information about the application that emits the record or about the infrastructure where the application runs.
    /// Data formats that represent this data model may be designed in a manner that allows the Resource field to be recorded only once per batch of log records that come from the same source.
    /// SHOULD follow [OpenTelemetry semantic conventions for Resources](https://opentelemetry.io/docs/specs/semconv/resource/).
    #[serde(flatten, with = "prefix_resource")]
    pub resource: Option<OTELResource>,

    /// Request trace id as defined in W3C Trace Context. Can be set for logs that are part of request processing and have an assigned trace id.
    /// Should be a sequence of bytes, encoded as a hexadecimal string
    /// This field is optional
    #[serde(rename = "trace_id")]
    pub trace_id_hex: Option<String>,

    /// Span id. Can be set for logs that are part of a particular processing span.
    /// If SpanId is present TraceId SHOULD be also present.
    /// Should be a sequence of bytes, encoded as a hexadecimal string
    /// This field is optional
    #[serde(rename = "span_id")]
    pub span_id_hex: Option<String>,

    /// Trace flag as defined in W3C Trace Context specification. At the time of writing the specification defines one flag - the SAMPLED flag.
    /// This field is optional
    pub trace_flags: Option<u32>,

    /// Additional information about the specific event occurrence.
    /// Unlike the Resource field, which is fixed for a particular source, Attributes can vary for each occurrence of the event coming from the same source.
    /// Can contain information about the request context (other than Trace Context Fields).
    pub attributes: Option<serde_json::Map<String, Value>>,

    pub dropped_attributes_count: u32,

    /// the instrumentation scope. Multiple occurrences of events coming from the same scope can happen across time and they all have the same value of InstrumentationScope
    #[serde(flatten, with = "prefix_scope")]
    pub scope: Option<OTELInstrumentationScope>,
}

impl OTELLogDocument {
    pub fn from_proto(
        scope: Option<OTELInstrumentationScope>,
        resource: Option<OTELResource>,
        record: LogRecord,
    ) -> Self {
        OTELLogDocument {
            resource,
            scope,
            timestamp_unix: Some(record.time_unix_nano as i64).filter(|ts| *ts > 0),
            observed_timestamp_unix: record.observed_time_unix_nano as i64,
            severity_text: Some(record.severity_text).filter(|t| !t.is_empty()),
            severity_number: Some(record.severity_number).filter(|s| *s > 0),
            body: record
                .body
                .and_then(otel_anyvalue_to_json)
                .map(|value| match value {
                    Value::Object(obj) => obj,
                    value => {
                        let mut obj = serde_json::Map::new();
                        obj.insert("message".to_string(), value);
                        obj
                    }
                }),
            trace_id_hex: Some(record.trace_id)
                .filter(|t| !t.is_empty())
                .map(hex::encode),
            span_id_hex: Some(record.span_id)
                .filter(|t| !t.is_empty())
                .map(hex::encode),
            trace_flags: Some(record.flags),
            attributes: Some(otel_attributes_to_json(record.attributes)),
            dropped_attributes_count: record.dropped_attributes_count,
        }
    }
}

#[inline]
pub fn otel_attributes_to_json(attributes: Vec<KeyValue>) -> serde_json::Map<String, Value> {
    attributes
        .into_iter()
        .filter_map(|kv| Some((kv.key, kv.value.and_then(otel_anyvalue_to_json)?)))
        .collect()
}

#[inline]
pub fn otel_anyvalue_to_json(value: AnyValue) -> Option<Value> {
    match value.value {
        Some(any_value::Value::StringValue(v)) => Some(Value::String(v)),
        Some(any_value::Value::BoolValue(v)) => Some(Value::Bool(v)),
        Some(any_value::Value::IntValue(v)) => Some(Value::Number(Number::from(v))),
        Some(any_value::Value::DoubleValue(v)) => Some(Value::Number(Number::from_f64(v)?)),
        Some(any_value::Value::ArrayValue(arr)) => Some(Value::Array(
            arr.values
                .into_iter()
                .filter_map(otel_anyvalue_to_json)
                .collect(),
        )),
        Some(any_value::Value::KvlistValue(kv_list)) => {
            Some(Value::Object(otel_attributes_to_json(kv_list.values)))
        }
        // We do not support indexing bytes at the moment
        Some(any_value::Value::BytesValue(_)) => None,
        None => None,
    }
}
