use async_trait::async_trait;
use url::Url;

use crate::domain::ports::file_storage::{FileStorage, FileStorageKey};

#[derive(Clone)]
pub struct FileStorageMock;


#[async_trait]
impl FileStorage for FileStorageMock {
    async fn store_file(&self, _file_storage_key: FileStorageKey, _content_type: &str, _data: Vec<u8>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_file(&self, key: FileStorageKey) -> anyhow::Result<Vec<u8>> {
        anyhow::bail!("Not implemented")
    }

    /// Returns a presigned URL for the file
    async fn get_file_url(&self, key: FileStorageKey) -> anyhow::Result<Url> {
        anyhow::bail!("Not implemented")
    }
}
