use std::{collections::HashMap, time::Duration};

use tracing::{error, warn};
use anyhow::Context;
use tonic::transport::Channel;

use crate::{
    application::application_config::AppConfig,
    domain::{
        entities::http_monitor::HttpMonitorErrorKind,
        ports::http_client::{HttpClient, PingResponse},
    },
    protos::{browser_client::BrowserClient, HttpErrorKind, HttpRequest}
};

#[derive(Clone)]
pub struct HttpClientAdapter {
    client: BrowserClient<Channel>,
}

impl HttpClientAdapter {
    pub async fn new(config: &AppConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(config.http_monitors_executor.browser_service_grpc_address.clone()).context("Invalid browser service grpc address")?;
        let client = BrowserClient::connect(channel).await.context("Failed to connect to browser service")?;
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl HttpClient for HttpClientAdapter {
    async fn ping(
        &self,
        endpoint: &str,
        request_timeout: Duration,
    ) -> PingResponse {
        let mut client = self.client.clone();
        let mut attempt = 0;
        loop {
            attempt += 1;
            let request = HttpRequest {
                endpoint: endpoint.to_string(),
                request_timeout_ms: request_timeout.as_millis() as u64,
                http_headers: HashMap::new(),
            };
            match client.execute_http_request(request).await {
                Ok(response) => {
                    let response = response.into_inner();
                    
                    // TODO: handle screenshots and other data
                    return PingResponse {
                        http_code: response.http_code.map(|code| code as u16),
                        error_kind: response.error.and_then(|kind| HttpErrorKind::try_from(kind).ok()).map(|kind| kind.into()).unwrap_or_default(),
                    };
                }
                Err(e) => {
                    if attempt >= 3 {
                        error!("Failed to call gRPC browser service: {:?}. Giving up.", e);
                        return PingResponse {
                            error_kind: HttpMonitorErrorKind::BrowserServiceCallFailed,
                            ..Default::default()
                        };
                    }
                    warn!("Failed to call gRPC browser service: {:?}. Retrying ...", e);
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    
                }
            }
        }
    }
}
