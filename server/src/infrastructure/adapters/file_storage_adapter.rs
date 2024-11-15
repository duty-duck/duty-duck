use std::time::Duration;

use async_trait::async_trait;
use aws_config::{AppName, BehaviorVersion};
use aws_sdk_s3::{presigning::PresigningConfig, primitives::ByteStream, Client};
use url::Url;
use uuid::Uuid;

use crate::domain::ports::file_storage::{FileStorage, FileStorageKey};

#[derive(Clone)]
pub struct FileStorageAdapter {
    client: Client,
    bucket_name: String,
}

impl FileStorageAdapter {
    pub async fn new(bucket_name: String) -> anyhow::Result<Self> {
        let app_name = AppName::new("duty-duck-server")?;
        let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let mut builder = aws_sdk_s3::config::Builder::from(&sdk_config);
        builder.set_app_name(Some(app_name));
        let client = aws_sdk_s3::Client::from_conf(builder.build());

        Ok(Self {
            client,
            bucket_name,
        })
    }
}

#[async_trait]
impl FileStorage for FileStorageAdapter {
    async fn store_file(&self, key: FileStorageKey, content_type: &str, data: Vec<u8>) -> anyhow::Result<()> {
        let body = ByteStream::from(data);
        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(key.to_string())
            .content_type(content_type)
            .body(body)
            .send()
            .await?;
        Ok(())
    }

    async fn get_file(&self, key: FileStorageKey) -> anyhow::Result<Vec<u8>> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket_name)
            .key(key.to_string())
            .send()
            .await?;
        Ok(response.body.collect().await?.to_vec())
    }

    async fn get_file_url(&self, key: FileStorageKey) -> anyhow::Result<Url> {
        let presigned_req = self
            .client
            .get_object()
            .bucket(&self.bucket_name)
            .key(key.to_string())
            .presigned(
                PresigningConfig::builder()
                    .expires_in(Duration::from_secs(30))
                    .build()?,
            )
            .await?;
        let presigned_url = Url::parse(presigned_req.uri())?;
        Ok(presigned_url)
    }
}
