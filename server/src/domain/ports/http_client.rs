use crate::domain::entities::http_monitor::HttpMonitorErrorKind;
use async_trait::async_trait;
use std::time::Duration;

#[derive(Default)]
pub struct PingResponse {
    pub http_code: Option<u16>,
    pub error_kind: HttpMonitorErrorKind,
}

#[async_trait]
pub trait HttpClient: Clone + Send + Sync + 'static {
    async fn ping(
        &self,
        endpoint: &str,
        request_timeout: Duration,
    ) -> PingResponse;
}
