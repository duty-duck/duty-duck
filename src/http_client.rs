use std::time::Duration;

use reqwest::{Client, Response};
use thiserror::Error;

use crate::app_env::AppConfig;

const MAX_BODY_SIZE: usize = 10_000_000;

pub struct HttpClient {
    client: reqwest::Client,
}

#[repr(u8)]
#[derive(Error, Debug)]
pub enum PingError {
    #[error("unknown error")]
    Unknown = 0,
    #[error("invalid HTTP status: {http_code:?}")]
    HttpCode { http_code: u16 } = 1,
    #[error("connection error")]
    Connect = 2,
    #[error("builder error")]
    Builder = 3,
    #[error("request error")]
    Request = 4,
    #[error("error following a redirect")]
    Redirect = 5,
    #[error("request or response body error")]
    Body = 6,
    #[error("error decoding response body")]
    Decode = 7,
    #[error("request timeout")]
    Timeout = 8,
}

impl HttpClient {
    pub fn new(config: &AppConfig) -> Self {
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .build()
            .unwrap();
        Self { client }
    }

    pub async fn ping_http_endpoint(
        &self,
        endpoint: &str,
        request_timeout: Duration,
    ) -> Result<(), PingError> {
        let result = self
            .client
            .head(endpoint)
            .timeout(request_timeout)
            .send()
            .await;

        match result {
            Ok(mut response) if response.status().is_client_error() || response.status().is_server_error() => {
                read_body_to_end_and_discard(&mut response).await;
                Err(PingError::HttpCode {
                    http_code: response.status().as_u16(),
                })
            }
            Ok(mut response) => {
                read_body_to_end_and_discard(&mut response).await;
                Ok(())
            }
            Err(e) if e.is_body() => Err(PingError::Body),
            Err(e) if e.is_builder() => Err(PingError::Builder),
            Err(e) if e.is_decode() => Err(PingError::Decode),
            Err(e) if e.is_connect() => Err(PingError::Connect),
            Err(e) if e.is_request() => Err(PingError::Request),
            Err(e) if e.is_redirect() => Err(PingError::Redirect),
            Err(e) if e.is_timeout() => Err(PingError::Timeout),
            Err(_) => Err(PingError::Unknown),
        }
    }
}

/// Consumes and discards the response body, up to [MAX_BODY_SIZE] bytes
async fn read_body_to_end_and_discard(response: &mut Response) {
    let mut read_bytes = 0;
    loop {
        match response.chunk().await {
            Ok(Some(b)) => {
                read_bytes += b.len();

                if read_bytes >= MAX_BODY_SIZE {
                    break;
                }
            }
            _ => {
                break;
            }
        }
    }
}
