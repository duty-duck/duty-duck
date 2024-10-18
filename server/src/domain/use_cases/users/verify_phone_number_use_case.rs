use anyhow::Context;
use serde::Deserialize;
use thiserror::Error;
use ts_rs::TS;

use crate::domain::{
    entities::{authorization::AuthContext, user::{UpdateUserCommand, UserPhoneOTP}},
    ports::{sms_notification_server::{Sms, SmsNotificationServer}, user_repository::UserRepository},
};

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct VerifyPhoneNumberCommand {
    pub code: String,
}

#[derive(Debug, Error)]
pub enum VerifyPhoneNumberError {
    #[error("User not found")]
    UserNotFound,
    #[error("Phone number not found")]
    PhoneNumberNotFound,
    #[error("Invalid code")]
    InvalidCode,
    #[error("Phone number already verified")]
    AlreadyVerified,
    #[error("Failed to verify phone number: {0}")]
    TechnicalFailure(#[from] anyhow::Error),
}

pub async fn verify_phone_number(
    auth_context: &AuthContext,
    repository: &impl UserRepository,
    command: VerifyPhoneNumberCommand,
) -> Result<(), VerifyPhoneNumberError> {
    let user = repository
        .get_user(auth_context.active_user_id, false)
        .await?
        .ok_or(VerifyPhoneNumberError::UserNotFound)?;
    if user.phone_number_verified {
        return Err(VerifyPhoneNumberError::AlreadyVerified);
    }
    let otp = user
        .phone_number_otp
        .ok_or(VerifyPhoneNumberError::InvalidCode)?;
    if otp.code != command.code {
        return Err(VerifyPhoneNumberError::InvalidCode);
    }

    repository
        .update_user(
            auth_context.active_user_id,
            UpdateUserCommand {
                phone_number_verified: Some(true),
                ..Default::default()
            },
        )
        .await.context("Failed to update user")?;
    Ok(())
}


pub async fn send_phone_number_verification_code(
    auth_context: &AuthContext,
    repository: &impl UserRepository,
    sms_notifications_server: &impl SmsNotificationServer
) -> Result<(), VerifyPhoneNumberError> {
    let user = repository
        .get_user(auth_context.active_user_id, false)
        .await?
        .ok_or(VerifyPhoneNumberError::UserNotFound)?;
    let phone_number = user.phone_number.ok_or(VerifyPhoneNumberError::PhoneNumberNotFound)?;
    let otp = UserPhoneOTP::new(phone_number.clone());
    let sms = Sms {
        phone_number,
        message: t!("smsPhoneNumberVerificationCode", code = otp.code).to_string()
    };

    sms_notifications_server.send_sms(&sms).await.context("Failed to send SMS")?;
    Ok(())
}
