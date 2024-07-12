use async_trait::async_trait;
/// A base trait for repositories that have transactions
#[async_trait]
pub trait TransactionalRepository {
    type Transaction: Send;

    async fn begin_transaction(&self) -> anyhow::Result<Self::Transaction>;
    async fn rollback_transaction(&self, tx: Self::Transaction) -> anyhow::Result<()>;
    async fn commit_transaction(&self, tx: Self::Transaction) -> anyhow::Result<()>;
}