use serde_json::{json, Value};

include!(concat!(env!("OUT_DIR"), "/built.rs"));

pub fn build_info_json() -> Value {
    json!({
        "gitVersion": GIT_VERSION,
        "gitCommitHash": GIT_COMMIT_HASH,
        "buildTime": BUILT_TIME_UTC
    })
}