use serde_json::Value;

use super::*;

/// The doc mapping of indices to store OTEL Logs, adapted from [Quickwit's documentation](https://quickwit.io/docs/log-management/otel-service#opentelemetry-logs-data-model)
/// and derived from ÓTEL's data model
pub fn otel_logs_doc_mapping() -> DocMapping {
    DocMapping {
        mode: DocMappingMode::Strict,
        partition_key: None,
        max_num_partitions: None,
        tag_fields: None,
        store_source: false,
        timestamp_field: Some("timestamp_nanos".to_string()),
        field_mappings: vec![
            FieldMapping::DateTime(DateTimeFieldMapping {
                name: "timestamp_nanos".to_string(),
                description: None,
                input_formats: Some(vec!["unix_timestamp".to_string()]),
                output_format: Some("unix_timestamp_nanos".to_string()),
                indexed: false,
                fast: true,
                fast_precision: Some(DateTimeFastPrecision::MilliSeconds),
                stored: true,
            }),
            FieldMapping::DateTime(DateTimeFieldMapping {
                name: "observed_timestamp_nanos".to_string(),
                description: None,
                input_formats: Some(vec!["unix_timestamp".to_string()]),
                output_format: Some("unix_timestamp_nanos".to_string()),
                indexed: true,
                stored: true,
                fast: false,
                fast_precision: None,
            }),
            FieldMapping::Text(TextFieldMapping {
                name: "severity_text".to_string(),
                description: None,
                stored: true,
                indexed: true,
                tokenizer: Some("raw".to_string()),
                record: None,
                fieldnorms: false,
                fast: true,
            }),
            FieldMapping::Unsigned64(NumericFieldMapping {
                name: "severity_number".to_string(),
                description: None,
                stored: true,
                indexed: true,
                fast: true,
                coerce: true,
            }),
            FieldMapping::Json(JsonFieldMapping {
                name: "body".to_string(),
                description: None,
                stored: true,
                indexed: true,
                fast: false,
                tokenizer: Some("default".to_string()),
                record: Some(Record::Position),
                expand_dots: true,
            }),
            FieldMapping::Unsigned64(NumericFieldMapping {
                name: "dropped_attributes_count".to_string(),
                description: None,
                stored: true,
                indexed: false,
                fast: false,
                coerce: true,
            }),
            FieldMapping::Bytes(BytesFieldMapping {
                name: "trace_id".to_string(),
                description: None,
                stored: true,
                indexed: true,
                fast: true,
                input_format: BytesEncoding::Hex,
                output_format: BytesEncoding::Hex,
            }),
            FieldMapping::Bytes(BytesFieldMapping {
                name: "span_id".to_string(),
                description: None,
                stored: true,
                indexed: true,
                fast: true,
                input_format: BytesEncoding::Hex,
                output_format: BytesEncoding::Hex,
            }),
            FieldMapping::Unsigned64(NumericFieldMapping {
                name: "trace_flags".to_string(),
                description: None,
                stored: true,
                indexed: false,
                fast: false,
                coerce: true,
            }),
            FieldMapping::Json(JsonFieldMapping {
                name: "resource_attributes".to_string(),
                description: None,
                stored: true,
                indexed: true,
                fast: true,
                tokenizer: Some("raw".to_string()),
                record: Some(Record::Basic),
                expand_dots: true,
            }),
            FieldMapping::Unsigned64(NumericFieldMapping {
                name: "resource_dropped_attributes_count".to_string(),
                description: None,
                stored: true,
                indexed: false,
                fast: false,
                coerce: true,
            }),
            FieldMapping::Text(TextFieldMapping {
                name: "scope_name".to_string(),
                description: None,
                indexed: false,
                stored: true,
                fast: false,
                tokenizer: None,
                fieldnorms: false,
                record: None,
            }),
            FieldMapping::Text(TextFieldMapping {
                name: "scope_version".to_string(),
                description: None,
                indexed: false,
                stored: true,
                fast: false,
                tokenizer: None,
                fieldnorms: false,
                record: None,
            }),
            FieldMapping::Json(JsonFieldMapping {
                name: "scope_attributes".to_string(),
                description: None,
                stored: true,
                indexed: false,
                fast: false,
                tokenizer: None,
                record: None,
                expand_dots: true,
            }),
            FieldMapping::Unsigned64(NumericFieldMapping {
                name: "scope_dropped_attributes_count".to_string(),
                description: None,
                stored: true,
                indexed: false,
                fast: false,
                coerce: true,
            }),
            FieldMapping::Json(JsonFieldMapping {
                name: "attributes".to_string(),
                description: None,
                stored: true,
                indexed: false,
                fast: false,
                tokenizer: None,
                record: None,
                expand_dots: true,
            }),
        ],
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct OTELResource {
    pub attributes: serde_json::Map<String, Value>,
    pub dropped_attributes_count: u32,
}

#[derive(Serialize, Debug, Clone)]
pub struct OTELInstrumentationScope {
    pub name: String,
    pub version: String,
    pub attributes: serde_json::Map<String, Value>,
    pub dropped_attributes_count: u32,
}

serde_with::with_prefix!(prefix_resource "resource_");
serde_with::with_prefix!(prefix_scope "scope_");

/// Represents the input document for a single ligne of log, as per the [otel_logs_doc_mapping]
#[derive(Serialize, Debug, Default, Clone)]
pub struct OTELLogInput {
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
