use anyhow::Context;
use chrono::Utc;
use tracing::warn;

use crate::domain::{
    entities::{
        http_monitor::{HttpMonitor, HttpMonitorStatus},
        incident::{
            HttpMonitorIncidentCause, HttpMonitorIncidentCausePing, Incident, IncidentCause,
            IncidentPriority, IncidentSource, IncidentStatus, NewIncident,
        },
        incident_event::{
            IncidentEvent, IncidentEventPayload, IncidentEventType, PingEventPayload,
        },
        incident_notification::IncidentNotificationPayload,
    },
    ports::{
        http_client::HttpClient,
        http_monitor_repository::{HttpMonitorRepository, UpdateHttpMonitorStatusCommand},
        incident_event_repository::IncidentEventRepository,
        incident_notification_repository::IncidentNotificationRepository,
        incident_repository::{IncidentRepository, ListIncidentsOpts},
    },
    use_cases::incidents::{
        confirm_incident, create_incident, resolve_incident, NotificationOpts,
    },
};

use super::{status_machine, ExecuteHttpMonitorsUseCase};

impl<HMR, IR, IER, INR, HC> ExecuteHttpMonitorsUseCase<HMR, IR, IER, INR, HC>
where
    HMR: HttpMonitorRepository,
    IR: IncidentRepository<Transaction = HMR::Transaction>,
    IER: IncidentEventRepository<Transaction = HMR::Transaction>,
    INR: IncidentNotificationRepository<Transaction = HMR::Transaction>,
    HC: HttpClient,
{
    ///
    /// Handles the result of pinging an HTTP monitor and updates the monitor's status accordingly.
    ///
    /// # Arguments
    /// * `transaction` - The database transaction to use for any updates
    /// * `monitor` - The HTTP monitor that was pinged
    /// * `ping_response` - The response from pinging the monitor, containing error info and HTTP code
    /// * `existing_incident` - Any existing incident associated with this monitor
    ///
    /// # Returns
    /// * `Ok(())` if the ping result was handled successfully
    /// * `Err` if there was an error updating the monitor status or handling incidents
    ///
    /// # Details
    /// This method:
    /// 1. Determines the next monitor status based on the ping result
    /// 2. Updates the monitor's status and related fields in the database
    /// 3. Creates/updates incidents if needed based on the monitor's new status
    pub async fn handle_ping_response(
        &self,
        transaction: &mut HMR::Transaction,
        mut monitor: HttpMonitor,
        ping_response: crate::domain::ports::http_client::PingResponse,
        existing_incident: Option<Incident>,
    ) -> anyhow::Result<()> {
        let (status_counter, status) = status_machine::next_status(
            monitor.downtime_confirmation_threshold,
            monitor.recovery_confirmation_threshold,
            monitor.status,
            monitor.status_counter,
            ping_response.error_kind
                == crate::domain::entities::http_monitor::HttpMonitorErrorKind::None,
        );

        let error_kind = ping_response.error_kind;
        let last_http_code = ping_response.http_code.map(|c| c as i16);
        let next_ping_at = Some(Utc::now() + monitor.interval());
        let last_status_change_at = if status != monitor.status {
            Utc::now()
        } else {
            monitor.last_status_change_at
        };
        let patch = UpdateHttpMonitorStatusCommand {
            organization_id: monitor.organization_id,
            monitor_id: monitor.id,
            last_http_code,
            status,
            status_counter,
            next_ping_at,
            error_kind,
            last_status_change_at,
        };

        // Update the monitor so these info will be used to create the incident
        monitor.error_kind = error_kind;
        monitor.last_http_code = last_http_code;

        // Update the monitor (status and status_counter)
        self.http_monitor_repository
            .update_http_monitor_status(transaction, patch)
            .await
            .with_context(|| "Failed to update HTTP monitor status")?;

        match (status, existing_incident) {
            // the monitor can never be unknown or inactive when we are handling a ping response
            (HttpMonitorStatus::Unknown | HttpMonitorStatus::Inactive, _) => unreachable!(),
            // a monitor is not supposed to transition to recovering without an incident
            // however, it may happen as a result of an incident being deleted from the database, and if it happens, we don't want to panic as it would block the entire system,
            // so we log a warning and do nothing more
            (HttpMonitorStatus::Recovering, None) => {
                warn!("Monitor is transitioning to recovering but no incident exists");
            }
            // if the monitor is up and no incident exists, we do nothing
            (HttpMonitorStatus::Up, None) => (),
            // if the monitor is up and there is an ongoing incident,
            // we create a ping event
            // we don't need to update the incident cause
            (HttpMonitorStatus::Up, Some(incident)) => {
                let ping_event = IncidentEvent {
                    organization_id: monitor.organization_id,
                    user_id: None,
                    created_at: Utc::now(),
                    incident_id: incident.id,
                    event_type: IncidentEventType::MonitorPinged,
                    event_payload: Some(IncidentEventPayload::MonitorPing(PingEventPayload {
                        error_kind: monitor.error_kind,
                        http_code: monitor.last_http_code.map(|c| c as i32),
                        http_headers: ping_response.http_headers,
                        response_time_ms: ping_response.response_time.as_millis() as u64,
                        response_ip_address: ping_response.response_ip_address,
                        resolved_ip_addresses: ping_response.resolved_ip_addresses,
                    })),
                };

                self.incident_event_repository
                    .create_incident_event(transaction, ping_event)
                    .await?;

                resolve_incident(
                    transaction,
                    &self.incident_repository,
                    &self.incident_event_repository,
                    &self.incident_notification_repository,
                    &incident,
                )
                .await?;
            }
            // if the monitor transitions to recovering for the first time, we create a ping event
            (HttpMonitorStatus::Recovering, Some(incident)) => {
                if status_counter == 1 {
                    let ping_event = IncidentEvent {
                        organization_id: monitor.organization_id,
                        user_id: None,
                        created_at: Utc::now(),
                        incident_id: incident.id,
                        event_type: IncidentEventType::MonitorPinged,
                        event_payload: Some(IncidentEventPayload::MonitorPing(PingEventPayload {
                            error_kind: monitor.error_kind,
                            http_code: monitor.last_http_code.map(|c| c as i32),
                            http_headers: ping_response.http_headers,
                            response_time_ms: ping_response.response_time.as_millis() as u64,
                            response_ip_address: ping_response.response_ip_address,
                            resolved_ip_addresses: ping_response.resolved_ip_addresses,
                        })),
                    };

                    self.incident_event_repository
                        .create_incident_event(transaction, ping_event)
                        .await?;
                }
            }
            // if the monitor is suspicious and there is no ongoing incident, we need to create a new unconfirmed incident
            (HttpMonitorStatus::Suspicious, None) => {
                self.create_incident_for_monitor(
                    transaction,
                    &monitor,
                    false,
                    IncidentCause::HttpMonitorIncidentCause(HttpMonitorIncidentCause {
                        last_ping: HttpMonitorIncidentCausePing {
                            error_kind: monitor.error_kind,
                            http_code: monitor.last_http_code,
                        },
                        previous_pings: vec![],
                    }),
                    ping_response,
                )
                .await?;
            }
            // if the monitor is down and there is no ongoing incident, we need to create a new confirmed incident
            (HttpMonitorStatus::Down, None) => {
                self.create_incident_for_monitor(
                    transaction,
                    &monitor,
                    true,
                    IncidentCause::HttpMonitorIncidentCause(HttpMonitorIncidentCause {
                        last_ping: HttpMonitorIncidentCausePing {
                            error_kind: monitor.error_kind,
                            http_code: monitor.last_http_code,
                        },
                        previous_pings: vec![],
                    }),
                    ping_response,
                )
                .await?;
            }
            // if the monitor is down and there is an unconfirmed incident, we need to confirm the incident
            // we may also need to update the incident cause if it has changed and create a ping event
            (
                HttpMonitorStatus::Down,
                Some(
                    ref incident @ Incident {
                        status: IncidentStatus::ToBeConfirmed,
                        cause: Some(IncidentCause::HttpMonitorIncidentCause(ref cause)),
                        ..
                    },
                ),
            ) => {
                self.confirm_incident_for_monitor(transaction, &monitor, incident)
                    .await?;

                if cause.last_ping.error_kind != monitor.error_kind
                    || cause.last_ping.http_code != monitor.last_http_code
                {
                    self.handle_changing_incident_cause(
                        transaction,
                        &monitor,
                        incident,
                        cause,
                        ping_response,
                    )
                    .await?;
                }
            }
            // if the monitor is down or suspicious and there is is an ongoing incident, we may need to update the incident to reflect the new cause
            // and create a ping event
            (
                HttpMonitorStatus::Down | HttpMonitorStatus::Suspicious,
                Some(
                    ref incident @ Incident {
                        cause: Some(IncidentCause::HttpMonitorIncidentCause(ref cause)),
                        ..
                    },
                ),
            ) => {
                if cause.last_ping.error_kind != monitor.error_kind
                    || cause.last_ping.http_code != monitor.last_http_code
                {
                    self.handle_changing_incident_cause(
                        transaction,
                        &monitor,
                        incident,
                        cause,
                        ping_response,
                    )
                    .await?;
                }
            },
            // if the monitor is down or suspicious and the cause of the incident is empty or not an http monitor incident cause, we do nothing
            // this should never happen, but we do not want to panic if it does
            (HttpMonitorStatus::Down | HttpMonitorStatus::Suspicious, Some(Incident { cause: _, .. })) => (),
        };

        Ok(())
    }

    /// Returns the existing ongoing incident for the given monitor
    pub(super) async fn get_existing_incident_for_monitor(
        &self,
        transaction: &mut IR::Transaction,
        monitor: &HttpMonitor,
    ) -> anyhow::Result<Option<Incident>>
    where
        IR: IncidentRepository,
    {
        let incident = self
            .incident_repository
            .list_incidents(
                transaction,
                monitor.organization_id,
                ListIncidentsOpts {
                    include_statuses: &[IncidentStatus::Ongoing, IncidentStatus::ToBeConfirmed],
                    include_priorities: &IncidentPriority::ALL,
                    include_sources: &[IncidentSource::HttpMonitor { id: monitor.id }],
                    limit: 1,
                    ..Default::default()
                },
            )
            .await?
            .incidents
            .into_iter()
            .next();
        Ok(incident)
    }

    /// Creates a new incident for the given monitor
    /// The incident is created in the same transaction as the monitor update.
    /// Returns the id of the created incident
    async fn create_incident_for_monitor(
        &self,
        transaction: &mut IR::Transaction,
        monitor: &HttpMonitor,
        confirmed_incident: bool,
        incident_cause: IncidentCause,
        ping_response: crate::domain::ports::http_client::PingResponse,
    ) -> anyhow::Result<()>
    where
        IR: IncidentRepository,
        IER: IncidentEventRepository<Transaction = IR::Transaction>,
        INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
    {
        let mut metadata = monitor.metadata.clone();
        if let Some(http_code) = monitor.last_http_code {
            metadata
                .records
                .insert("http_code".to_string(), http_code.to_string());
        }
        if let Ok(url) = monitor.url() {
            metadata.records.insert("url".to_string(), url.to_string());
            if let Some(host) = url.host_str() {
                metadata
                    .records
                    .insert("host".to_string(), host.to_string());
            }
        }

        let new_incident = NewIncident {
            organization_id: monitor.organization_id,
            created_by: None,
            status: if confirmed_incident {
                IncidentStatus::Ongoing
            } else {
                IncidentStatus::ToBeConfirmed
            },
            // TODO: let users configure this
            priority: IncidentPriority::Major,
            source: IncidentSource::HttpMonitor { id: monitor.id },
            cause: Some(incident_cause.clone()),
            metadata: monitor.metadata.clone(),
        };

        // send a notification only if the incident is confirmed
        let notification = if confirmed_incident {
            Some(NotificationOpts {
                send_sms: monitor.sms_notification_enabled,
                send_push_notification: monitor.push_notification_enabled,
                send_email: monitor.email_notification_enabled,
                notification_payload: IncidentNotificationPayload {
                    incident_cause,
                    incident_http_monitor_url: Some(monitor.url.clone()),
                },
            })
        } else {
            None
        };

        let incident_id = create_incident(
            transaction,
            &self.incident_repository,
            &self.incident_event_repository,
            &self.incident_notification_repository,
            new_incident,
            notification,
        )
        .await?;

        let ping_event = IncidentEvent {
            incident_id,
            event_type: IncidentEventType::MonitorPinged,
            event_payload: Some(IncidentEventPayload::MonitorPing(PingEventPayload {
                error_kind: monitor.error_kind,
                http_code: monitor.last_http_code.map(|c| c as i32),
                http_headers: ping_response.http_headers,
                response_time_ms: ping_response.response_time.as_millis() as u64,
                response_ip_address: ping_response.response_ip_address,
                resolved_ip_addresses: ping_response.resolved_ip_addresses,
            })),
            organization_id: monitor.organization_id,
            user_id: None,
            created_at: Utc::now(),
        };
        self.incident_event_repository
            .create_incident_event(transaction, ping_event)
            .await?;

        Ok(())
    }

    /// Confirms an incident for the given monitor
    /// The incident is confirmed in the same transaction as the monitor update.
    async fn confirm_incident_for_monitor(
        &self,
        transaction: &mut IR::Transaction,
        monitor: &HttpMonitor,
        incident: &Incident,
    ) -> anyhow::Result<()>
    where
        IR: IncidentRepository,
        IER: IncidentEventRepository<Transaction = IR::Transaction>,
        INR: IncidentNotificationRepository<Transaction = IR::Transaction>,
    {
        let notification = NotificationOpts {
            send_sms: monitor.sms_notification_enabled,
            send_push_notification: monitor.push_notification_enabled,
            send_email: monitor.email_notification_enabled,
            notification_payload: IncidentNotificationPayload {
                incident_cause: incident
                    .cause
                    .clone()
                    .context("Incident cause is required")?,
                incident_http_monitor_url: Some(monitor.url.clone()),
            },
        };

        confirm_incident(
            transaction,
            &self.incident_repository,
            &self.incident_event_repository,
            &self.incident_notification_repository,
            incident,
            notification,
        )
        .await
    }

    async fn handle_changing_incident_cause(
        &self,
        transaction: &mut IR::Transaction,
        monitor: &HttpMonitor,
        incident: &Incident,
        cause: &HttpMonitorIncidentCause,
        ping_response: crate::domain::ports::http_client::PingResponse,
    ) -> anyhow::Result<()> {
        let mut previous_pings = cause.previous_pings.clone();
        previous_pings.push(cause.last_ping.clone());

        let cause = IncidentCause::HttpMonitorIncidentCause(HttpMonitorIncidentCause {
            last_ping: HttpMonitorIncidentCausePing {
                error_kind: monitor.error_kind,
                http_code: monitor.last_http_code,
            },
            previous_pings,
        });

        let updated_incident = Incident {
            cause: Some(cause),
            ..incident.clone()
        };

        let ping_event = IncidentEvent {
            organization_id: monitor.organization_id,
            user_id: None,
            created_at: Utc::now(),
            incident_id: incident.id,
            event_type: IncidentEventType::MonitorPinged,
            event_payload: Some(IncidentEventPayload::MonitorPing(PingEventPayload {
                error_kind: monitor.error_kind,
                http_code: monitor.last_http_code.map(|c| c as i32),
                http_headers: ping_response.http_headers,
                response_time_ms: ping_response.response_time.as_millis() as u64,
                response_ip_address: ping_response.response_ip_address,
                resolved_ip_addresses: ping_response.resolved_ip_addresses,
            })),
        };

        self.incident_repository
            .update_incident(transaction, updated_incident)
            .await?;
        self.incident_event_repository
            .create_incident_event(transaction, ping_event)
            .await?;

        Ok(())
    }
}
