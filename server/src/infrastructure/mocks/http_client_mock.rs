use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::domain::ports::http_client::{HttpClient, PingResponse};

#[derive(Clone)]
pub struct HttpClientMock {
    pub next_response: Arc<Mutex<Option<PingResponse>>>,
}

impl HttpClientMock {
    pub fn new() -> Self {
        Self {
            next_response: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_next_response(&self, response: PingResponse) {
        *self.next_response.lock().await = Some(response);
    }
}

#[async_trait]
impl HttpClient for HttpClientMock {
    async fn ping(
        &self,
        _endpoint: &str,
        _request_timeout: Duration,
        _request_headers: HashMap<String, String>,
    ) -> PingResponse {
        let mut next_response = self.next_response.lock().await;
        next_response.take().unwrap()
    }
}
