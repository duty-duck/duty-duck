use crate::domain::entities::http_monitor::HttpMonitorErrorKind;
use async_trait::async_trait;
use std::{collections::HashMap, time::Duration};

#[derive(Debug, Clone)]
pub struct Screenshot {
    pub data: Vec<u8>,
    pub content_type: String,
}

#[derive(Debug, Default, Clone)]
pub struct PingResponse {
    pub http_code: Option<u16>,
    pub error_kind: HttpMonitorErrorKind,
    pub http_headers: HashMap<String, String>,
    pub response_time: Duration,
    pub response_ip_address: Option<String>,
    pub resolved_ip_addresses: Vec<String>,
    pub response_body_size_bytes: u64,
    pub response_body_content: Option<Vec<u8>>,
    pub screenshot: Option<Screenshot>
}

#[async_trait]
pub trait HttpClient: Clone + Send + Sync + 'static {
    async fn ping(
        &self,
        endpoint: &str,
        request_timeout: Duration,
    ) -> PingResponse;
}
