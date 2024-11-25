-- Add down migration script here
alter table http_monitors
    drop column email_notification_enabled,
    drop column push_notification_enabled,
    drop column sms_notification_enabled;
