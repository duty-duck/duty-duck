use std::env;

use anyhow::Context;
use lettre::{
    message::{header::ContentType, MessageBuilder},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

#[derive(Clone)]
pub struct Mailer {
    inner: AsyncSmtpTransport<Tokio1Executor>,
}

impl Mailer {
    pub fn new() -> anyhow::Result<Self> {
        let host = env::var("SMTP_SERVER_HOST").unwrap_or("localhost".to_string());
        let port = match env::var("SMTP_SERVER_PORT") {
            Ok(var) => var
                .parse::<u16>()
                .with_context(|| "Failed to parse SMTP_SERVER_PORT")?,
            _ => 25,
        };
        let skip_tls = env::var("SMTP_SKIP_TLS") == Ok("true".to_string());
        Ok(Self {
            inner: if skip_tls {
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&host)
                    .port(port)
                    .build()
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&host)?
                    .port(port)
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
