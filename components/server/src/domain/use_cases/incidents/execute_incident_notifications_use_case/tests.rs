use std::{path::PathBuf, str::FromStr};

use chrono::Utc;
use uuid::Uuid;

use crate::{
    application::templates::Templates,
    domain::entities::{
        incident::{
            HttpMonitorIncidentCause, HttpMonitorIncidentCausePing, IncidentCause,
            ScheduledTaskIncidentCause, TaskRunIncidentCause,
        },
        incident_notification::{
            IncidentNotification, IncidentNotificationPayload, IncidentNotificationType,
        },
        organization::Organization,
        task::TaskUserId,
        user::{PreferredCommunicationLanguage, User},
    },
    infrastructure::{
        adapters::{
            mailer_adapter::MailerAdapter,
            organization_repository_adapter::OrganizationRepositoryAdapter,
            push_notification_server_adapter::PushNotificationServerAdapter,
            sms_notification_server_adapter::SmsNotificationServerAdapter,
            user_devices_repository_adapter::UserDevicesRepositoryAdapter,
        },
        mocks::{
            incident_event_repository_mock::IncidentEventRepositoryMock,
            incident_notification_repository_mock::IncidentNotificationRepositoryMock,
        },
    },
};

use super::ExecuteIncidentNotificationsUseCase;

type UseCase = ExecuteIncidentNotificationsUseCase<
    OrganizationRepositoryAdapter,
    IncidentNotificationRepositoryMock,
    IncidentEventRepositoryMock,
    PushNotificationServerAdapter,
    SmsNotificationServerAdapter,
    UserDevicesRepositoryAdapter,
    MailerAdapter,
>;

#[test]
fn test_build_email_message() {
    let templates = Templates::new(PathBuf::from_str("templates").unwrap()).unwrap();
    let languages = [
        PreferredCommunicationLanguage::En,
        PreferredCommunicationLanguage::Fr,
    ];
    let user = User {
        id: Uuid::new_v4(),
        first_name: "Jane".to_string(),
        last_name: "Doe".to_string(),
        email: "foo@bar.com".to_string(),
        phone_number: None,
        phone_number_verified: false,
        phone_number_otp: None,
        preferred_communicaton_language: PreferredCommunicationLanguage::En,
    };
    let org = Organization {
        id: Uuid::new_v4(),
        name: "Foo Inc.".to_string(),
        display_name: "Foo Inc".to_string(),
        stripe_customer_id: None,
        billing_address: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let notifications = [
        IncidentNotification {
            organization_id: org.id,
            incident_id: Uuid::new_v4(),
            escalation_level: 0,
            notification_type: IncidentNotificationType::IncidentCreation,
            notification_due_at: Utc::now(),
            notification_payload: IncidentNotificationPayload {
                incident_cause: IncidentCause::HttpMonitorIncidentCause(HttpMonitorIncidentCause {
                    last_ping: HttpMonitorIncidentCausePing {
                        error_kind:
                            crate::domain::entities::http_monitor::HttpMonitorErrorKind::Timeout,
                        http_code: None,
                    },
                    previous_pings: Default::default(),
                }),
                incident_http_monitor_url: Some("https://foo.com".to_string()),
                incident_task_id: None,
            },
            send_sms: false,
            send_push_notification: false,
            send_email: false,
        },
        IncidentNotification {
            organization_id: org.id,
            incident_id: Uuid::new_v4(),
            escalation_level: 0,
            notification_type: IncidentNotificationType::IncidentCreation,
            notification_due_at: Utc::now(),
            notification_payload: IncidentNotificationPayload {
                incident_cause: IncidentCause::TaskRunIncidentCause(TaskRunIncidentCause {
                    task_id: TaskUserId::new("Foo").unwrap(),
                    task_run_id: Uuid::new_v4(),
                    task_run_started_at: Utc::now(),
                    task_run_finished_at: None,
                    task_run_status: crate::domain::entities::task_run::TaskRunStatus::Dead,
                }),
                incident_http_monitor_url: None,
                incident_task_id: Some(TaskUserId::new("Foo").unwrap()),
            },
            send_sms: false,
            send_push_notification: false,
            send_email: false,
        },
        IncidentNotification {
            organization_id: org.id,
            incident_id: Uuid::new_v4(),
            escalation_level: 0,
            notification_type: IncidentNotificationType::IncidentCreation,
            notification_due_at: Utc::now(),
            notification_payload: IncidentNotificationPayload {
                incident_cause: IncidentCause::ScheduledTaskIncidentCause(
                    ScheduledTaskIncidentCause {
                        task_id: Uuid::new_v4(),
                        task_user_id: TaskUserId::new("Foo").unwrap(),
                        task_was_due_at: Utc::now(),
                        task_ran_late_at: None,
                        task_switched_to_absent_at: None,
                    },
                ),
                incident_http_monitor_url: None,
                incident_task_id: Some(TaskUserId::new("Foo").unwrap()),
            },
            send_sms: false,
            send_push_notification: false,
            send_email: false,
        },
    ];

    for lang in languages {
        for notification in &notifications {
            let result = UseCase::build_email_message(
                &templates,
                notification,
                &User {
                    preferred_communicaton_language: lang,
                    ..user.clone()
                },
                &org,
            );
            assert!(
                result.is_ok(),
                "failed to build e-mail message for lang {lang}: {:#?}",
                result.err().unwrap()
            );
        }
    }
}

#[test]
fn test_build_sms_message() {
    let languages = [
        PreferredCommunicationLanguage::En,
        PreferredCommunicationLanguage::Fr,
    ];
    let user = User {
        id: Uuid::new_v4(),
        first_name: "Jane".to_string(),
        last_name: "Doe".to_string(),
        email: "foo@bar.com".to_string(),
        phone_number: Some("+36600000000".to_string()),
        phone_number_verified: false,
        phone_number_otp: None,
        preferred_communicaton_language: PreferredCommunicationLanguage::En,
    };
    let org = Organization {
        id: Uuid::new_v4(),
        name: "Foo Inc.".to_string(),
        display_name: "Foo Inc".to_string(),
        stripe_customer_id: None,
        billing_address: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let notifications = [
        IncidentNotification {
            organization_id: org.id,
            incident_id: Uuid::new_v4(),
            escalation_level: 0,
            notification_type: IncidentNotificationType::IncidentCreation,
            notification_due_at: Utc::now(),
            notification_payload: IncidentNotificationPayload {
                incident_cause: IncidentCause::HttpMonitorIncidentCause(HttpMonitorIncidentCause {
                    last_ping: HttpMonitorIncidentCausePing {
                        error_kind:
                            crate::domain::entities::http_monitor::HttpMonitorErrorKind::Timeout,
                        http_code: None,
                    },
                    previous_pings: Default::default(),
                }),
                incident_http_monitor_url: Some("https://foo.com".to_string()),
                incident_task_id: None,
            },
            send_sms: false,
            send_push_notification: false,
            send_email: false,
        },
        IncidentNotification {
            organization_id: org.id,
            incident_id: Uuid::new_v4(),
            escalation_level: 0,
            notification_type: IncidentNotificationType::IncidentCreation,
            notification_due_at: Utc::now(),
            notification_payload: IncidentNotificationPayload {
                incident_cause: IncidentCause::TaskRunIncidentCause(TaskRunIncidentCause {
                    task_id: TaskUserId::new("Foo").unwrap(),
                    task_run_id: Uuid::new_v4(),
                    task_run_started_at: Utc::now(),
                    task_run_finished_at: None,
                    task_run_status: crate::domain::entities::task_run::TaskRunStatus::Dead,
                }),
                incident_http_monitor_url: None,
                incident_task_id: Some(TaskUserId::new("Foo").unwrap()),
            },
            send_sms: false,
            send_push_notification: false,
            send_email: false,
        },
        IncidentNotification {
            organization_id: org.id,
            incident_id: Uuid::new_v4(),
            escalation_level: 0,
            notification_type: IncidentNotificationType::IncidentCreation,
            notification_due_at: Utc::now(),
            notification_payload: IncidentNotificationPayload {
                incident_cause: IncidentCause::ScheduledTaskIncidentCause(
                    ScheduledTaskIncidentCause {
                        task_id: Uuid::new_v4(),
                        task_user_id: TaskUserId::new("Foo").unwrap(),
                        task_was_due_at: Utc::now(),
                        task_ran_late_at: None,
                        task_switched_to_absent_at: None,
                    },
                ),
                incident_http_monitor_url: None,
                incident_task_id: Some(TaskUserId::new("Foo").unwrap()),
            },
            send_sms: false,
            send_push_notification: false,
            send_email: false,
        },
    ];

    for lang in languages {
        for notification in &notifications {
            let result = UseCase::build_sms_message(
                notification,
                &User {
                    preferred_communicaton_language: lang,
                    ..user.clone()
                },
                &org,
            );
            assert!(
                result.is_ok(),
                "failed to build SMS message for lang {lang}: {:#?}",
                result.err().unwrap()
            );
        }
    }
}

#[test]
fn test_build_push_notification() {
    let notifications = [];

    for notification in &notifications {
        let result = UseCase::build_push_notification(notification);
        assert!(
            result.is_ok(),
            "failed to build push notification: {:#?}",
            result.err().unwrap()
        );
    }
}
