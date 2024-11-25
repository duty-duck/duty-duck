use aws_config::{AppName, BehaviorVersion};
use aws_sdk_sns::{types::MessageAttributeValue, Client};

use crate::domain::ports::sms_notification_server::*;

#[derive(Clone)]
pub struct SmsNotificationServerAdapter {
    sns_client: Client,
}

impl SmsNotificationServerAdapter {
    pub async fn new() -> anyhow::Result<Self> {
        let app_name = AppName::new("duty-duck-server")?;
        let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let mut builder = aws_sdk_sns::config::Builder::from(&sdk_config);
        builder.set_app_name(Some(app_name));
        let sns_client = aws_sdk_sns::Client::from_conf(builder.build());

        Ok(Self { sns_client })
    }
}

#[async_trait::async_trait]
impl SmsNotificationServer for SmsNotificationServerAdapter {
    #[tracing::instrument(skip(self))]
    async fn send_sms(
        &self,
        Sms {
            phone_number,
            message,
        }: &Sms,
    ) -> anyhow::Result<()> {
        self.sns_client
            .publish()
            .phone_number(phone_number)
            .message(message)
            .message_attributes("AWS.SNS.SMS.SMSType", MessageAttributeValue::builder().data_type("String").string_value("Transactional").build()?)
            .send()
            .await?;

        Ok(())
    }
}
