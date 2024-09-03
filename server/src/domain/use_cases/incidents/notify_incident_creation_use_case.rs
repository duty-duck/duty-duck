use std::{collections::HashMap, time::Duration};

use tokio::task::JoinSet;
use tracing::*;
use uuid::Uuid;

use crate::domain::{
    entities::{
        incident::{IncidentSourceWithDetails, IncidentWithSourcesDetails},
        push_notification::PushNotification,
        user_device::UserDevice,
    },
    ports::{
        incident_notification_repository::IncidentNotificationRepository,
        push_notification_server::PushNotificationServer,
        user_devices_repository::UserDevicesRepository,
    },
};

pub async fn spawn_new_incident_notification_tasks<INR, PNS, UDR>(
    n_tasks: usize,
    delay_between_two_executions: Duration,
    incident_notification_repository: INR,
    push_notificaton_server: PNS,
    user_devices_repository: UDR,
    select_limit: u32,
) -> JoinSet<()>
where
    INR: IncidentNotificationRepository,
    PNS: PushNotificationServer,
    UDR: UserDevicesRepository,
{
    let mut join_set = JoinSet::new();

    for _ in 0..n_tasks {
        let mut interval = tokio::time::interval(delay_between_two_executions);
        let incident_notification_repository = incident_notification_repository.clone();
        let push_notificaton_server = push_notificaton_server.clone();
        let user_devices_repository = user_devices_repository.clone();

        join_set.spawn(async move {
            loop {
                let _ = interval.tick().await;
                match fetch_due_incidents_and_notify_incident_creation(
                    &incident_notification_repository,
                    &push_notificaton_server,
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

async fn fetch_due_incidents_and_notify_incident_creation<INR, PNS, UDR>(
    incident_notification_repository: &INR,
    push_notificaton_server: &PNS,
    user_devices_repository: &UDR,
    select_limit: u32,
) -> anyhow::Result<usize>
where
    INR: IncidentNotificationRepository,
    PNS: PushNotificationServer,
    UDR: UserDevicesRepository,
{
    let mut user_devices_by_organization: HashMap<Uuid, Vec<UserDevice>> = HashMap::new();
    let mut tx = incident_notification_repository.begin_transaction().await?;
    let incidents = incident_notification_repository
        .list_new_incidents_due_for_notification(&mut tx, select_limit)
        .await?;

    let incidents_len = incidents.len();
    for incident in incidents {
        let org_id = incident.incident.organization_id;
        // List devices for the notification
        // Right now, all the devices in the organization are notified
        // In the future, we want to add on-call management so that only some users are notified and incidents can be escalated
        let org_user_devices = match user_devices_by_organization.get(&org_id) {
            Some(devices) => devices,
            None => {
                let devices = user_devices_repository
                    .list_organization_devices(org_id)
                    .await?;
                user_devices_by_organization
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

        // Save in the database that the notification has been properly sent
        incident_notification_repository
            .acknowledge_incident_creation_notification(&mut tx, org_id, incident.incident.id)
            .await?;
    }

    incident_notification_repository
        .commit_transaction(tx)
        .await?;

    Ok(incidents_len)
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
