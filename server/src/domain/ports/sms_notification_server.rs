use futures::{stream::FuturesUnordered, StreamExt};
use veil::Redact;

#[async_trait::async_trait]
pub trait SmsNotificationServer: Clone + Send + Sync + 'static {
    async fn send_sms(&self, message: &Sms) -> anyhow::Result<()>;

    async fn send_batch(&self, messages: Vec<Sms>) -> anyhow::Result<()> {
        messages
            .into_iter()
            .map(|message| async move { self.send_sms(&message).await })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(())
    }
}

#[derive(Clone, Redact)]
pub struct Sms {
    #[redact(partial)]
    pub phone_number: String,
    pub message: String,
}
