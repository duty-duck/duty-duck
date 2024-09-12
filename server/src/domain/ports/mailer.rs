use lettre::{message::MessageBuilder, Message};

#[async_trait::async_trait]
pub trait Mailer: Clone + Send + Sync + 'static {
    async fn send(&self, message: Message) -> anyhow::Result<()>;

    async fn send_batch(&self, messages: Vec<Message>) -> anyhow::Result<()>;

    fn builder() -> MessageBuilder;
}
