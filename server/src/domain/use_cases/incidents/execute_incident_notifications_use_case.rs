use std::{collections::HashMap, time::Duration};

use anyhow::Context;
use chrono::Utc;
use lettre::Message;
use tokio::task::JoinSet;
use tracing::*;
use uuid::Uuid;

use crate::domain::{
    entities::{
        incident::IncidentCause, incident_event::{IncidentEvent, IncidentEventPayload, IncidentEventType, NotificationEventPayload}, incident_notification::IncidentNotification, organization::Organization, push_notification::{PushNotification, PushNotificationToken}, user::User, user_device::UserDevice
    },
    ports::{
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository, mailer::Mailer,
        organization_repository::OrganizationRepository,
        push_notification_server::PushNotificationServer,
        user_devices_repository::UserDevicesRepository,
    },
};

#[allow(clippy::too_many_arguments)]
pub fn spawn_tasks<OR, INR, IER, PNS, UDR, M>(
    n_tasks: usize,
    delay_between_two_executions: Duration,
    organization_repository: OR,
    incident_notification_repository: INR,
    incident_event_repository: IER,
    push_notificaton_server: PNS,
    mailer: M,
    user_devices_repository: UDR,
    select_limit: u32,
) -> JoinSet<()>
where
    OR: OrganizationRepository,
    INR: IncidentNotificationRepository,
    IER: IncidentEventRepository<Transaction = INR::Transaction>,
    PNS: PushNotificationServer,
    UDR: UserDevicesRepository,
    M: Mailer,
{
    let mut join_set = JoinSet::new();

    for _ in 0..n_tasks {
        let mut interval = tokio::time::interval(delay_between_two_executions);
        let incident_notification_repository = incident_notification_repository.clone();
        let push_notificaton_server = push_notificaton_server.clone();
        let user_devices_repository = user_devices_repository.clone();
        let organization_repository = organization_repository.clone();
        let incident_event_repository = incident_event_repository.clone();
        let mailer = mailer.clone();

        join_set.spawn(async move {
            loop {
                let _ = interval.tick().await;
                match fetch_and_execute_due_notifications(
                    &organization_repository,
                    &incident_notification_repository,
                    &incident_event_repository,
                    &push_notificaton_server,
                    &mailer,
                    &user_devices_repository,
                    select_limit,
                )
                .await
                {
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
        });
    }

    join_set
}

async fn fetch_and_execute_due_notifications<M, OR, INR, IER, PNS, UDR>(
    org_repository: &OR,
    incident_notification_repository: &INR,
    incident_event_repository: &IER,
    push_notificaton_server: &PNS,
    mailer: &M,
    user_devices_repository: &UDR,
    select_limit: u32,
) -> anyhow::Result<usize>
where
    OR: OrganizationRepository,
    INR: IncidentNotificationRepository,
    IER: IncidentEventRepository<Transaction = INR::Transaction>,
    PNS: PushNotificationServer,
    UDR: UserDevicesRepository,
    M: Mailer,
{
    let mut user_devices_cache: UserDevicesByOrgCache = UserDevicesByOrgCache::new();
    let mut org_cache: OrgCache = OrgCache::new();

    let mut tx = incident_notification_repository.begin_transaction().await?;
    let incident_notifications = incident_notification_repository
        .get_next_notifications_to_send(&mut tx, select_limit)
        .await?;

    let incident_notifications_len = incident_notifications.len();
    debug!(
        notifications = incident_notifications_len,
        "{} incident notifications are due to be sent", incident_notifications_len
    );

    for notification in incident_notifications {
        let should_create_event = notification.send_email || notification.send_push_notification || notification.send_sms;
        let event = IncidentEvent {
            organization_id: notification.organization_id,
            incident_id: notification.incident_id,
            created_at: Utc::now(),
            user_id: None,
            event_type: IncidentEventType::Notification,
            event_payload: Some(IncidentEventPayload::Notification(NotificationEventPayload {
                escalation_level: notification.escalation_level,
                sent_via_email: notification.send_email,
                sent_via_push_notification: notification.send_push_notification,
                sent_via_sms: notification.send_sms,
            })),
        };

       send_notification::<M>(
            notification,
            user_devices_repository,
            org_repository,
            push_notificaton_server,
            mailer,
            &mut user_devices_cache,
            &mut org_cache,
        )
        .await?;

        if should_create_event {
            incident_event_repository.create_incident_event(&mut tx, event).await?;
        }
    }

    // Commit the transaction.
    // Once the transaction is committed, the due notifications are deleted from the database
    incident_notification_repository
        .commit_transaction(tx)
        .await?;

    Ok(incident_notifications_len)
}

/// Sends an event notification, if any notification channel is enabled
async fn send_notification<M: Mailer>(
    notification: IncidentNotification,
    user_devices_repository: &impl UserDevicesRepository,
    org_repository: &impl OrganizationRepository,
    push_notificaton_server: &impl PushNotificationServer,
    mailer: &M,
    user_devices_cache: &mut UserDevicesByOrgCache,
    org_cache: &mut OrgCache,
) -> anyhow::Result<()> {
    let org_id = notification.organization_id;
    let (org, org_users) = fetch_organization_and_users(org_repository, org_id, org_cache).await?;

    // Send e-mails, if e-email notifications are enabled
    if notification.send_email {
        let messages = org_users
            .into_iter()
            .filter_map(
                |user| match build_message::<M>(&notification, &user, &org) {
                    Ok(message) => Some(message),
                    Err(e) => {
                        warn!(error = ?e, user = ?user, "Failed to build e-mail message for user");
                        None
                    }
                },
            )
            .collect();
        mailer.send_batch(messages).await?;
    }

    // Send push notification, if push notifications are enabled
    if notification.send_push_notification {
        let devices_tokens =
            fetch_organization_devices_token(user_devices_repository, user_devices_cache, org_id)
                .await?;

        match build_push_notification(&notification) {
            Ok(push_notification) => {
                push_notificaton_server
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
fn build_message<M: Mailer>(
    notification: &IncidentNotification,
    user: &User,
    _user_org: &Organization,
) -> anyhow::Result<Message> {
    let subject;
    let body;

    match &notification.notification_payload.incident_cause {
        IncidentCause::HttpMonitorIncidentCause {
            ..
        } => {
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
    org_repository: &impl OrganizationRepository,
    org_id: Uuid,
    cache: &mut OrgCache,
) -> anyhow::Result<(Organization, Vec<User>)> {
    if let Some(cached_data) = cache.get(&org_id) {
        return Ok(cached_data.clone());
    }
    let mut users = Vec::new();
    let organization = org_repository
        .get_organization(org_id)
        .await
        .with_context(|| format!("Failed to fetch organization with id: {}", org_id))?;

    loop {
        let page_size = 100;
        let page_results = org_repository
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
    user_devices_repository: &impl UserDevicesRepository,
    user_devices_cache: &mut UserDevicesByOrgCache,
    org_id: Uuid,
) -> anyhow::Result<Vec<PushNotificationToken>> {
    let org_user_devices = match user_devices_cache.get(&org_id) {
        Some(devices) => devices,
        None => {
            let devices = user_devices_repository
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

type UserDevicesByOrgCache = HashMap<Uuid, Vec<UserDevice>>;
type OrgCache = HashMap<Uuid, (Organization, Vec<User>)>;
