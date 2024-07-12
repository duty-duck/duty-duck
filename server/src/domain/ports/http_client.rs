use std::time::Duration;
use async_trait::async_trait;
use crate::domain::entities::http_monitor::HttpMonitorErrorKind;

pub struct PingResponse {
    pub http_code: u16
}

pub struct PingError {
    pub http_code: Option<u16>,
    pub error_kind: HttpMonitorErrorKind

}

#[async_trait]
pub trait HttpClient: Clone + Send + Sync + 'static {
    async fn ping(&self, endpoint: &str, request_timeout: Duration) -> Result<PingResponse, PingError>;
}