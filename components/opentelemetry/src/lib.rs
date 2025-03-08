pub mod json;

pub mod proto {
    use opentelemetry_proto::tonic::common::v1::{
        any_value, AnyValue, ArrayValue, KeyValue, KeyValueList,
    };
    pub use opentelemetry_proto::tonic::*;
    use serde_json::Value;

    #[inline]
    pub fn json_to_anyvalue(value: Value) -> AnyValue {
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
                            value: Some(json_to_anyvalue(v)),
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
                    values: arr.into_iter().map(json_to_anyvalue).collect(),
                })),
            },
        }
    }
}
