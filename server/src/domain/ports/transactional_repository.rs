use async_trait::async_trait;

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionMock;

/// A base trait for repositories that have transactions
#[async_trait]
pub trait TransactionalRepository {
    type Transaction: Send;

    async fn begin_transaction(&self) -> anyhow::Result<Self::Transaction>;
    async fn rollback_transaction(&self, tx: Self::Transaction) -> anyhow::Result<()>;
    async fn commit_transaction(&self, tx: Self::Transaction) -> anyhow::Result<()>;
}

/// A macro than can be used to implement [TransactionalRepository] automatically for any struct containing
/// a [sqlx::PgPool] named `pool`
#[macro_export]
macro_rules! postgres_transactional_repo {
    ($t:ident) => {
        #[async_trait::async_trait]
        impl $crate::domain::ports::transactional_repository::TransactionalRepository for $t {
            type Transaction = sqlx::Transaction<'static, sqlx::Postgres>;

            async fn begin_transaction(&self) -> anyhow::Result<Self::Transaction> {
                use ::anyhow::*;
                self.pool
                    .begin()
                    .await
                    .with_context(|| "Cannot begin transaction")
            }

            async fn rollback_transaction(&self, tx: Self::Transaction) -> anyhow::Result<()> {
                use ::anyhow::*;
                tx.rollback()
                    .await
                    .with_context(|| "Cannot rollback transaction")
            }

            async fn commit_transaction(&self, tx: Self::Transaction) -> anyhow::Result<()> {
                use ::anyhow::*;
                tx.commit()
                    .await
                    .with_context(|| "Cannot commit transaction")
            }
        }
    };
}
