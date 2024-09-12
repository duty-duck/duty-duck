use std::{collections::HashMap, time::Duration};

use tokio::task::JoinSet;
use tracing::*;
use uuid::Uuid;

use crate::domain::{
    entities::{
        incident::{IncidentSourceWithDetails, IncidentWithSourcesDetails},
        push_notification::PushNotification,
        user::User,
        user_device::UserDevice,
    },
    ports::{
        incident_notification_repository::IncidentNotificationRepository,
        mailer::Mailer,
        push_notification_server::PushNotificationServer,
        user_devices_repository::UserDevicesRepository,
        user_repository::{self, UserRepository},
    },
};

#[allow(clippy::too_many_arguments)]
pub async fn spawn_new_incident_notification_tasks(
    n_tasks: usize,
    delay_between_two_executions: Duration,
    user_repository: impl UserRepository,
    incident_notification_repository: impl IncidentNotificationRepository,
    push_notificaton_server: impl PushNotificationServer,
    mailer: impl Mailer,
    user_devices_repository: impl UserDevicesRepository,
    select_limit: u32,
) -> JoinSet<()> {
    let mut join_set = JoinSet::new();

    for _ in 0..n_tasks {
        let mut interval = tokio::time::interval(delay_between_two_executions);
        let incident_notification_repository = incident_notification_repository.clone();
        let push_notificaton_server = push_notificaton_server.clone();
        let user_devices_repository = user_devices_repository.clone();
        let user_repository = user_repository.clone();
        let mailer = mailer.clone();

        join_set.spawn(async move {
            loop {
                let _ = interval.tick().await;
                match fetch_due_incidents_and_notify_incident_creation(
                    &user_repository,
                    &incident_notification_repository,
                    &push_notificaton_server,
                    &mailer,
                    &user_devices_repository,
                    select_limit,
                )
                .await
                {
                    Ok(incidents) if incidents > 0 => {
                        debug!(
                            incidents,
                            "Notified users of {} newly-created incidents", incidents
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

async fn fetch_due_incidents_and_notify_incident_creation(
    user_repository: &impl UserRepository,
    incident_notification_repository: &impl IncidentNotificationRepository,
    push_notificaton_server: &impl PushNotificationServer,
    mailer: &impl Mailer,
    user_devices_repository: &impl UserDevicesRepository,
    select_limit: u32,
) -> anyhow::Result<usize> {
    let mut user_devices_cache: UserDevicesByOrgCache = UserDevicesByOrgCache::new();
    let mut users_cache: UserByOrgCache = UserByOrgCache::new();

    let mut tx = incident_notification_repository.begin_transaction().await?;
    let incidents = incident_notification_repository
        .list_new_incidents_due_for_notification(&mut tx, select_limit)
        .await?;

    let incidents_len = incidents.len();
    for incident in incidents {
        proces_incident(
            &incident,
            user_devices_repository,
            push_notificaton_server,
            &mut user_devices_cache,
            &mut users_cache,
        )
        .await?;
        // Save in the database that the notification has been properly sent
        incident_notification_repository
            .acknowledge_incident_creation_notification(
                &mut tx,
                incident.incident.organization_id,
                incident.incident.id,
            )
            .await?;
    }

    incident_notification_repository
        .commit_transaction(tx)
        .await?;

    Ok(incidents_len)
}

async fn proces_incident(
    incident: &IncidentWithSourcesDetails,
    user_devices_repository: &impl UserDevicesRepository,
    push_notificaton_server: &impl PushNotificationServer,
    user_devices_cache: &mut UserDevicesByOrgCache,
    users_cache: &mut UserByOrgCache
) -> anyhow::Result<()> {
    let org_id = incident.incident.organization_id;
    // List devices for the notification
    // Right now, all the devices in the organization are notified
    // In the future, we want to add on-call management so that only some users are notified and incidents can be escalated
    let org_user_devices = match user_devices_cache.get(&org_id) {
        Some(devices) => devices,
        None => {
            let devices = user_devices_repository
                .list_organization_devices(org_id)
                .await?;
            user_devices_cache
                .entry(org_id)
                .or_insert(devices)
        }
    };
    let devices_tokens = org_user_devices
        .iter()
        .filter_map(|device| device.push_notification_token.0.clone())
        .collect::<Vec<_>>();

    let push_notification = build_notification(&incident);

    push_notificaton_server
        .send(&devices_tokens, &push_notification)
        .await?;

    Ok(())
}

fn build_notification(incident: &IncidentWithSourcesDetails) -> PushNotification {
    let mut notification = PushNotification {
        title: t!("newIncidentDefaultPushNotificationTitle").to_string(),
        body: t!("newIncidentDefaultPushNotificationBody").to_string(),
    };

    for source in &incident.sources {
        match source {
            IncidentSourceWithDetails::HttpMonitor { url, .. } => {
                notification.title =
                    t!("newHttpMonitorIncidentPushNotificationTitle", url = url).to_string();
                notification.body =
                    t!("newHttpMonitorIncidentPushNotificationBody", url = url).to_string();
            }
        }
    }

    notification
}

type UserDevicesByOrgCache = HashMap<Uuid, Vec<UserDevice>>;
type UserByOrgCache = HashMap<Uuid, Vec<User>>;
