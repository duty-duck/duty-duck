use veil::Redact;

#[async_trait::async_trait]
pub trait SmsNotificationServer {
    async fn send_sms(&self, message: &Sms) -> anyhow::Result<()>;
}

#[derive(Clone, Redact)]
pub struct Sms {
    #[redact(partial)]
    pub phone_number: String,
    pub message: String,
}
