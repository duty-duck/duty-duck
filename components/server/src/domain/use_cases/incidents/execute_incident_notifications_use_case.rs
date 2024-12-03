use std::{collections::HashMap, time::Duration};

use anyhow::Context;
use chrono::Utc;
use lettre::Message;
use tokio::task::JoinSet;
use tracing::*;
use uuid::Uuid;

use crate::domain::{
    entities::{
        incident::IncidentCause,
        incident_event::{
            IncidentEvent, IncidentEventPayload, IncidentEventType, NotificationEventPayload,
        },
        incident_notification::IncidentNotification,
        organization::Organization,
        push_notification::{PushNotification, PushNotificationToken},
        user::User,
        user_device::UserDevice,
    },
    ports::{
        incident_event_repository::IncidentEventRepository, incident_notification_repository::IncidentNotificationRepository, mailer::Mailer, organization_repository::OrganizationRepository, push_notification_server::PushNotificationServer, sms_notification_server::{Sms, SmsNotificationServer}, user_devices_repository::UserDevicesRepository
    },
};

#[derive(Clone)]
pub struct ExecuteIncidentNotificationsUseCase<OR, INR, IER, PNS, SNS, UDR, M> {
    pub organization_repository: OR,
    pub incident_notification_repository: INR,
    pub incident_event_repository: IER,
    pub push_notificaton_server: PNS,
    pub sms_notificaton_server: SNS,
    pub mailer: M,
    pub user_devices_repository: UDR,
    pub select_limit: u32,
}

impl<OR, INR, IER, PNS, SNS, UDR, M> ExecuteIncidentNotificationsUseCase<OR, INR, IER, PNS, SNS, UDR, M>
where
    OR: OrganizationRepository,
    INR: IncidentNotificationRepository,
    IER: IncidentEventRepository<Transaction = INR::Transaction>,
    PNS: PushNotificationServer,
    SNS: SmsNotificationServer,
    UDR: UserDevicesRepository,
    M: Mailer,
{
    pub fn spawn_tasks(
        &self,
        n_tasks: usize,
        delay_between_two_executions: Duration,
    ) -> JoinSet<()> {
        let mut join_set = JoinSet::new();

        for _ in 0..n_tasks {
            let executor = self.clone();

            join_set.spawn(async move {
                let mut interval = tokio::time::interval(delay_between_two_executions);
                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            match executor.fetch_and_execute_due_notifications().await {
                                Ok(notifications) if notifications > 0 => {
                                    info!(
                                        notifications,
                                        "Send {} incident notifications", notifications
                                    );
                                }
                                Err(e) => {
                                    error!(error = ?e, "Failed to notify users of new incidents")
                                }
                                Ok(_) => {}
                            }
                        }
                        _ = tokio::signal::ctrl_c() => {
                            info!("Shutting down incident notifications task");
                            break;
                        }
                    }
                }
            });
        }

        join_set
    }

    async fn fetch_and_execute_due_notifications(
        &self
    ) -> anyhow::Result<usize>
  {
        let mut user_devices_cache: UserDevicesByOrgCache = UserDevicesByOrgCache::new();
        let mut org_cache: OrgCache = OrgCache::new();

        let mut tx = self.incident_notification_repository.begin_transaction().await?;
        let incident_notifications = self
            .incident_notification_repository
            .get_next_notifications_to_send(&mut tx, self.select_limit)
            .await?;

        let incident_notifications_len = incident_notifications.len();
        debug!(
            notifications = incident_notifications_len,
            "{} incident notifications are due to be sent", incident_notifications_len
        );

        for notification in incident_notifications {
            let should_create_event = notification.send_email
                || notification.send_push_notification
                || notification.send_sms;
            let event = IncidentEvent {
                organization_id: notification.organization_id,
                incident_id: notification.incident_id,
                created_at: Utc::now(),
                user_id: None,
                event_type: IncidentEventType::Notification,
                event_payload: Some(IncidentEventPayload::Notification(
                    NotificationEventPayload {
                        escalation_level: notification.escalation_level,
                        sent_via_email: notification.send_email,
                        sent_via_push_notification: notification.send_push_notification,
                        sent_via_sms: notification.send_sms,
                    },
                )),
            };

            self.send_notification(
                notification,
                &mut user_devices_cache,
                &mut org_cache,
            )
            .await?;

            if should_create_event {
                self.incident_event_repository
                    .create_incident_event(&mut tx, event)
                    .await?;
            }
        }

        // Commit the transaction.
        // Once the transaction is committed, the due notifications are deleted from the database
        self.incident_notification_repository
            .commit_transaction(tx)
            .await?;

        Ok(incident_notifications_len)
    }

    /// Sends an event notification, if any notification channel is enabled
    async fn send_notification(
        &self,        
        notification: IncidentNotification,
        user_devices_cache: &mut UserDevicesByOrgCache,
        org_cache: &mut OrgCache,
    ) -> anyhow::Result<()> {
        let org_id = notification.organization_id;
        let (org, org_users) =
            self.fetch_organization_and_users(org_id, org_cache).await?;

        // Send e-mails, if e-email notifications are enabled
        if notification.send_email {
            let messages = org_users
            .iter()
            .filter_map(
                |user| match Self::build_email_message(&notification, user, &org) {
                    Ok(message) => Some(message),
                    Err(e) => {
                        warn!(error = ?e, user = ?user, "Failed to build e-mail message for user");
                        None
                    }
                },
            )
            .collect();
            self.mailer.send_batch(messages).await?;
        }

        // Send SMS, if SMS notifications are enabled
        if notification.send_sms {
            let messages = org_users
            .into_iter()
            .filter(|user| user.phone_number.is_some() && user.phone_number_verified)
            .filter_map(
                |user| match Self::build_sms_message(&notification, &user, &org) {
                    Ok(message) => Some(message),
                    Err(e) => {
                        warn!(error = ?e, user = ?user, "Failed to build SMS message for user");
                        None
                    }
                },
            )
            .collect();
            self.sms_notificaton_server.send_batch(messages).await?;
        }

        // Send push notification, if push notifications are enabled
        if notification.send_push_notification {
            let devices_tokens = self.fetch_organization_devices_token(
                user_devices_cache,
                org_id,
            )
            .await?;

            match Self::build_push_notification(&notification) {
                Ok(push_notification) => {
                    self.push_notificaton_server
                        .send(&devices_tokens, &push_notification)
                        .await?;
                }
                Err(e) => {
                    warn!(error = ?e, "Failed to build push notification");
                }
            }
        }

        Ok(())
    }

    /// Builds a push notification for an incident.
    ///
    /// This function creates a `PushNotification` based on the details of the given incident.
    /// It starts with a default notification message and then customizes it based on the
    /// incident's source type.
    ///
    /// # Arguments
    ///
    /// * `notification` - A reference to an `IncidentNotification` containing information about the incident.
    ///
    /// # Returns
    ///
    /// Returns a `PushNotification` struct with a title and body tailored to the incident.
    fn build_push_notification(
        notification: &IncidentNotification,
    ) -> anyhow::Result<PushNotification> {
        match &notification.notification_payload.incident_cause {
            IncidentCause::HttpMonitorIncidentCause { .. } => {
                let url = notification.notification_payload.incident_http_monitor_url.as_ref().context("Cannot build push notification, cause is HttpMonitorIncidentCause but HTTP monitor URL is not set")?;
                Ok(PushNotification {
                    title: t!("newHttpMonitorIncidentPushNotificationTitle", url = url).to_string(),
                    body: t!("newHttpMonitorIncidentPushNotificationBody", url = url).to_string(),
                })
            }
        }
    }

    /// Builds an email message for an incident.
    ///
    /// This function creates a `lettre::Message` based on the details of the given incident.
    /// It customizes the email subject and body based on the incident's source type.
    ///
    /// # Arguments
    ///
    /// * `notification` - A reference to an `IncidentNotification` containing information about the incident.
    /// * `recipient_email` - The email address of the recipient.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `lettre::Message` if successful, or an error if message building fails.
    fn build_email_message(
        notification: &IncidentNotification,
        user: &User,
        _user_org: &Organization,
    ) -> anyhow::Result<Message> {
        let subject;
        let body;

        match &notification.notification_payload.incident_cause {
            IncidentCause::HttpMonitorIncidentCause { .. } => {
                let url = notification.notification_payload.incident_http_monitor_url.as_ref().context("Cannot build e-mail message, cause is HttpMonitorIncidentCause but HTTP monitor URL is not set")?;
                subject = t!("newHttpMonitorIncidentEmailSubject", url = url).to_string();
                body = t!("newHttpMonitorIncidentEmailBody", url = url).to_string();
            }
        }

        M::builder()
            .to(user.email.parse()?)
            .subject(subject)
            .body(body)
            .with_context(|| "Failed to build message")
    }

    fn build_sms_message(
        notification: &IncidentNotification,
        user: &User,
        _user_org: &Organization,
    ) -> anyhow::Result<Sms> {
        match &notification.notification_payload.incident_cause {
            IncidentCause::HttpMonitorIncidentCause { .. } => {
                let url = notification.notification_payload.incident_http_monitor_url.as_ref().context("Cannot build push notification, cause is HttpMonitorIncidentCause but HTTP monitor URL is not set")?;
                Ok(Sms {
                    phone_number: user.phone_number.clone().context("Cannot build SMS message, user has no phone number")?,
                    message: t!("newHttpMonitorIncidentPushNotificationBody", url = url).to_string(),
                })
            }
        }
    }

    /// Fetches an organization and its users, using a cache if available.
    ///
    /// # Arguments
    ///
    /// * `org_repository` - The organization repository
    /// * `org_id` - The ID of the organization to fetch
    /// * `cache` - A mutable reference to the organization cache
    ///
    /// # Returns
    ///
    /// Returns a Result containing a tuple of the Organization and its Users.
    async fn fetch_organization_and_users(
        &self,
        org_id: Uuid,
        cache: &mut OrgCache,
    ) -> anyhow::Result<(Organization, Vec<User>)> {
        if let Some(cached_data) = cache.get(&org_id) {
            return Ok(cached_data.clone());
        }
        let mut users = Vec::new();
        let organization = self.organization_repository
            .get_organization(org_id)
            .await
            .with_context(|| format!("Failed to fetch organization with id: {}", org_id))?;

        loop {
            let page_size = 100;
            let page_results = self.organization_repository
                .list_organization_members(org_id, 0, page_size as u32)
                .await
                .with_context(|| {
                    format!(
                        "Failed to fetch members for organization with id: {}",
                        org_id
                    )
                })?;
            let page_results_len = page_results.len();
            users.extend(page_results);
            if page_results_len < page_size {
                break;
            }
        }

        let result = (organization, users);
        cache.insert(org_id, result.clone());

        Ok(result)
    }

    /// Fetches push notification tokens for devices in an organization.
    ///
    /// Uses a cache to improve performance and falls back to the repository if needed.
    ///
    /// # Arguments
    ///
    /// * `user_devices_repository` - The repository for user devices
    /// * `user_devices_cache` - A mutable reference to the user devices cache
    /// * `org_id` - The ID of the organization
    ///
    /// # Returns
    ///
    /// A Result containing a vector of PushNotificationTokens, or an error
    async fn fetch_organization_devices_token(
        &self,
        user_devices_cache: &mut UserDevicesByOrgCache,
        org_id: Uuid,
    ) -> anyhow::Result<Vec<PushNotificationToken>> {
        let org_user_devices = match user_devices_cache.get(&org_id) {
            Some(devices) => devices,
            None => {
                let devices = self.user_devices_repository
                    .list_organization_devices(org_id)
                    .await?;
                user_devices_cache.entry(org_id).or_insert(devices)
            }
        };
        let devices_tokens = org_user_devices
            .iter()
            .filter_map(|device| device.push_notification_token.0.clone())
            .collect::<Vec<_>>();
        Ok(devices_tokens)
    }
}
type UserDevicesByOrgCache = HashMap<Uuid, Vec<UserDevice>>;
type OrgCache = HashMap<Uuid, (Organization, Vec<User>)>;
