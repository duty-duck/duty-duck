use std::{collections::HashMap, time::Duration};

use anyhow::Context;
use tonic::transport::Channel;
use tracing::{error, warn};

use crate::{
    application::application_config::AppConfig,
    domain::{
        entities::http_monitor::HttpMonitorErrorKind,
        ports::http_client::{HttpClient, PingResponse, Screenshot},
    },
    protos::{browser_client::BrowserClient, HttpErrorKind, HttpRequest},
};

#[derive(Clone)]
pub struct HttpClientAdapter {
    client: BrowserClient<Channel>,
}

impl HttpClientAdapter {
    pub async fn new(config: &AppConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(
            config
                .http_monitors_executor
                .browser_service_grpc_address
                .clone(),
        )
        .context("Invalid browser service grpc address")?;
        let client = BrowserClient::connect(channel)
            .await
            .context("Failed to connect to browser service")?;
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl HttpClient for HttpClientAdapter {
    async fn ping(
        &self,
        endpoint: &str,
        request_timeout: Duration,
        request_headers: HashMap<String, String>,
    ) -> PingResponse {
        let mut client = self.client.clone();
        let mut attempt = 0;
        loop {
            attempt += 1;
            let request = HttpRequest {
                endpoint: endpoint.to_string(),
                request_timeout_ms: request_timeout.as_millis() as u64,
                http_headers: request_headers.clone(),
            };
            match client.execute_http_request(request).await {
                Ok(response) => {
                    let response = response.into_inner();

                    return PingResponse {
                        http_code: response.http_code.map(|code| code as u16),
                        error_kind: response
                            .error
                            .and_then(|kind| HttpErrorKind::try_from(kind).ok())
                            .map(|kind| kind.into())
                            .unwrap_or(HttpMonitorErrorKind::None),
                        http_headers: response.http_headers,
                        response_time: Duration::from_millis(response.response_time_ms),
                        response_ip_address: response.response_ip_address,
                        resolved_ip_addresses: response.resolved_ip_addresses,
                        response_body_size_bytes: response.response_body_size_bytes,
                        response_body_content: response.response_body_content,
                        screenshot: response.screenshot.map(|screenshot| Screenshot {
                            data: screenshot.data,
                            content_type: screenshot.content_type,
                        }),
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
