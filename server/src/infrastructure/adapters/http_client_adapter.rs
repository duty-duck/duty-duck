use std::time::Duration;

use crate::{
    application::application_config::AppConfig,
    domain::{
        entities::http_monitor::HttpMonitorErrorKind,
        ports::http_client::{HttpClient, PingError, PingResponse},
    },
};
const MAX_BODY_SIZE: usize = 2_000_000;

#[derive(Clone)]
pub struct HttpClientAdapter {
    client: reqwest::Client,
}

impl HttpClientAdapter {
    pub fn new(config: &AppConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(&config.user_agent)
            .build()
            .unwrap();
        Self { client }
    }

    /// Consumes and discards the response body, up to [MAX_BODY_SIZE] bytes
    async fn read_body_to_end_and_discard(response: &mut reqwest::Response) {
        let mut read_bytes = 0;
        while let Ok(Some(b)) = response.chunk().await {
            read_bytes += b.len();

            if read_bytes >= MAX_BODY_SIZE {
                break;
            }
        }
    }
}

#[async_trait::async_trait]
impl HttpClient for HttpClientAdapter {
    async fn ping(
        &self,
        endpoint: &str,
        request_timeout: Duration,
    ) -> Result<PingResponse, PingError> {
        let result = self
            .client
            .head(endpoint)
            .timeout(request_timeout)
            .send()
            .await;

        match result {
            Ok(mut response)
                if response.status().is_client_error() || response.status().is_server_error() =>
            {
                Self::read_body_to_end_and_discard(&mut response).await;
                Err(PingError {
                    http_code: Some(response.status().as_u16()),
                    error_kind: HttpMonitorErrorKind::HttpCode,
                })
            }
            Ok(mut response) => {
                Self::read_body_to_end_and_discard(&mut response).await;
                Ok(PingResponse {
                    http_code: response.status().as_u16(),
                })
            }
            Err(e) if e.is_body() => Err(PingError {
                http_code: None,
                error_kind: HttpMonitorErrorKind::Body,
            }),
            Err(e) if e.is_builder() => Err(PingError {
                http_code: None,
                error_kind: HttpMonitorErrorKind::Builder,
            }),
            Err(e) if e.is_decode() => Err(PingError {
                http_code: None,
                error_kind: HttpMonitorErrorKind::Decode,
            }),
            Err(e) if e.is_connect() => Err(PingError {
                http_code: None,
                error_kind: HttpMonitorErrorKind::Connect,
            }),
            Err(e) if e.is_request() => Err(PingError {
                http_code: None,
                error_kind: HttpMonitorErrorKind::Request,
            }),
            Err(e) if e.is_redirect() => Err(PingError {
                http_code: None,
                error_kind: HttpMonitorErrorKind::Redirect,
            }),
            Err(e) if e.is_timeout() => Err(PingError {
                http_code: None,
                error_kind: HttpMonitorErrorKind::Timeout,
            }),
            Err(_) => Err(PingError {
                http_code: None,
                error_kind: HttpMonitorErrorKind::Unknown,
            }),
        }
    }
}
