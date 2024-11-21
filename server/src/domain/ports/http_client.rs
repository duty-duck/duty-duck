use crate::domain::entities::http_monitor::HttpMonitorErrorKind;
use async_trait::async_trait;
use std::{collections::HashMap, time::Duration};

#[derive(Debug, Clone)]
pub struct Screenshot {
    pub data: Vec<u8>,
    pub content_type: String,
}

#[derive(Default, Clone)]
pub struct PingResponse {
    pub http_code: Option<u16>,
    pub error_kind: HttpMonitorErrorKind,
    pub http_headers: HashMap<String, String>,
    pub response_time: Duration,
    pub response_ip_address: Option<String>,
    pub resolved_ip_addresses: Vec<String>,
    #[allow(unused)]
    pub response_body_size_bytes: u64,
    pub response_body_content: Option<Vec<u8>>,
    pub screenshot: Option<Screenshot>
}

impl std::fmt::Debug for PingResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PingResponse")
            .field("http_code", &self.http_code)
            .field("error_kind", &self.error_kind)
            .field("http_headers_len", &self.http_headers.len())
            .field("response_ip_address", &self.response_ip_address)
            .field("has_screenshot", &self.screenshot.is_some())
            .finish()
    }
}

#[async_trait]
pub trait HttpClient: Clone + Send + Sync + 'static {
    async fn ping(
        &self,
        endpoint: &str,
        request_timeout: Duration,
        request_headers: HashMap<String, String>,
    ) -> PingResponse;
}
