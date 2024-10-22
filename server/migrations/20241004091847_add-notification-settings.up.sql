-- Add up migration script here
alter table http_monitors
    add column email_notification_enabled boolean not null default true,
    add column push_notification_enabled boolean not null default true,
    add column sms_notification_enabled boolean not null default false;