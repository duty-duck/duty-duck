-- Add up migration script here
create table user_devices (
    organization_id uuid not null,
    id uuid not null default gen_random_uuid(),
    user_id uuid not null,
    created_at timestamp with time zone not null default now(),
    label text not null,
    device_type smallint not null,
    push_notification_token text,
    primary key (organization_id, id)
);
create index on user_devices (organization_id, user_id);

-- create a table that will act as a "job queue" for incident notifications
create table incidents_notifications (
    organization_id uuid not null,
    incident_id uuid not null,
    escalation_level smallint not null default 0,
    -- 0: incident creation, 1: incident resolution
    notification_type smallint not null default 0,
    notification_due_at timestamp with time zone not null,
    notification_payload jsonb not null,
    send_sms boolean not null,
    send_push_notification boolean not null,
    send_email boolean not null,
    primary key (organization_id, incident_id, escalation_level),
    foreign key (organization_id, incident_id) references incidents (organization_id, id)
);
create index on incidents_notifications (notification_due_at desc);
