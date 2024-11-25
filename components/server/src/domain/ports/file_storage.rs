use std::fmt::Display;

use async_trait::async_trait;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct FileStorageKey {
    pub organization_id: Uuid,
    pub file_id: Uuid,
}

impl Display for FileStorageKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "storagev1/{}.{}", self.organization_id, self.file_id)
    }
}

#[async_trait]
pub trait FileStorage: Clone + Send + Sync + 'static {
    /// Stores the file data
    async fn store_file(
        &self,
        key: FileStorageKey,
        content_type: &str,
        data: Vec<u8>,
    ) -> anyhow::Result<()>;

    /// Returns the file data
    #[allow(unused)]
    async fn get_file(&self, key: FileStorageKey) -> anyhow::Result<Vec<u8>>;

    /// Returns a presigned URL for the file
    async fn get_file_url(&self, key: FileStorageKey) -> anyhow::Result<Url>;
}
