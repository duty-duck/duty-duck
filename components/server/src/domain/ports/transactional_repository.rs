use async_trait::async_trait;

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionMock;

/// A base trait for repositories that have transactions
#[async_trait]
pub trait TransactionalRepository {
    type Transaction: Send;

    async fn begin_transaction(&self) -> anyhow::Result<Self::Transaction>;

    #[allow(unused)]
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

            #[tracing::instrument(skip(self))]
            async fn begin_transaction(&self) -> anyhow::Result<Self::Transaction> {
                use ::anyhow::Context;

                tracing::trace!("Beginning SQL transaction");
                match self
                    .pool
                    .begin()
                    .await
                    .context("Cannot begin transaction")
                {
                    std::result::Result::Ok(tx) => std::result::Result::Ok(tx),
                    std::result::Result::Err(error) => {
                        tracing::error!(error = ?error, "Failed to begin SQL transaction: {:?}", error);
                        std::result::Result::Err(error)
                    },
                }
            }

            #[tracing::instrument(skip(self))]
            async fn rollback_transaction(&self, tx: Self::Transaction) -> anyhow::Result<()> {
                use ::anyhow::Context;
                tracing::trace!("Rollbacking SQL transaction");
                match tx.rollback()
                    .await
                    .context("Cannot rollback transaction") {
                        Err(error) => {
                            tracing::error!(error = ?error, "Failed to rollback SQL transaction: {:?}", error);
                            Err(error)
                        },
                        _ => Ok(())
                    }
            }

            #[tracing::instrument(skip(self))]
            async fn commit_transaction(&self, tx: Self::Transaction) -> anyhow::Result<()> {
                use ::anyhow::Context;
                tracing::trace!("Committing SQL transaction");
                match tx.commit()
                    .await
                    .context("Cannot commit transaction") {
                        Err(error) => {
                            tracing::error!(error = ?error, "Failed to commit SQL transaction: {:?}", error);
                            Err(error)
                        },
                        _ => Ok(())
                    }
            }
        }
    };
}
