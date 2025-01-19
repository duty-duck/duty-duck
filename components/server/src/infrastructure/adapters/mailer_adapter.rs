use anyhow::Context;
use futures::future::try_join_all;
use lettre::*;
use message::{header::ContentType, MessageBuilder};
use tracing::info;
use transport::smtp::authentication::{Credentials, Mechanism};

use crate::domain::ports::mailer::Mailer;

#[derive(Clone)]
pub struct MailerAdapter {
    inner: AsyncSmtpTransport<Tokio1Executor>,
}

pub struct MailerAdapterConfig {
    pub smtp_server_host: String,
    pub smtp_server_port: u16,
    pub smtp_disable_tls: bool,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
}

impl MailerAdapter {
    pub fn new(config: MailerAdapterConfig) -> anyhow::Result<Self> {
        info!(
            smtp_server_host = config.smtp_server_host,
            smtp_server_port = config.smtp_server_port,
            "Creating mailer adapter"
        );

        let mut builder = if config.smtp_disable_tls {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_server_host)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server_host)?
        };

        builder = builder.port(config.smtp_server_port);
        if let Some(username) = config.smtp_username {
            let password = config
                .smtp_password
                .with_context(|| "Cannot send SMTP username without a corresponding password")?;
            builder = builder
                .authentication(vec![Mechanism::Plain])
                .credentials(Credentials::new(username, password));
        }

        Ok(Self {
            inner: builder.build(),
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
