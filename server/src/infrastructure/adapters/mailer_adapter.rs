use futures::future::try_join_all;
use lettre::*;
use message::{header::ContentType, MessageBuilder};
use tracing::info;

use crate::domain::ports::mailer::Mailer;

#[derive(Clone)]
pub struct MailerAdapter {
    inner: AsyncSmtpTransport<Tokio1Executor>,
}

impl MailerAdapter {
    pub fn new(
        smtp_server_host: &str,
        smtp_server_port: u16,
        smtp_disable_tls: bool,
    ) -> anyhow::Result<Self> {
        info!(
            smtp_server_host,
            smtp_server_port, "Creating mailer adapter"
        );

        Ok(Self {
            inner: if smtp_disable_tls {
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(smtp_server_host)
                    .port(smtp_server_port)
                    .build()
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::relay(smtp_server_host)?
                    .port(smtp_server_port)
                    .build()
            },
        })
    }
}

#[async_trait::async_trait]
impl Mailer for MailerAdapter {
    async fn send(&self, message: Message) -> anyhow::Result<()> {
        self.inner.send(message).await?;
        Ok(())
    }

    async fn send_batch(&self, messages: Vec<Message>) -> anyhow::Result<()> {
        let futures = messages.into_iter().map(|m| self.send(m));
        try_join_all(futures).await?;
        Ok(())
    }

    fn builder() -> MessageBuilder {
        Message::builder()
            .from("Duty Duck <support@dutyduck.net>".parse().unwrap())
            .header(ContentType::TEXT_PLAIN)
    }
}
