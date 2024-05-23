use lettre::{
    message::{header::ContentType, MessageBuilder},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::app_env::AppConfig;

#[derive(Clone)]
pub struct Mailer {
    inner: AsyncSmtpTransport<Tokio1Executor>,
}

impl Mailer {
    pub fn new(config: &AppConfig) -> anyhow::Result<Self> {
        Ok(Self {
            inner: if config.smtp_disable_tls {
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_server_host)
                    .port(config.smtp_server_port)
                    .build()
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server_host)?
                    .port(config.smtp_server_port)
                    .build()
            },
        })
    }

    pub fn builder() -> MessageBuilder {
        Message::builder()
            .from("Duty Duck <support@dutyduck.com>".parse().unwrap())
            .header(ContentType::TEXT_HTML)
    }

    pub async fn send(&self, message: Message) -> anyhow::Result<()> {
        self.inner.send(message).await?;
        Ok(())
    }
}
