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
    creation_notification_sent_at timestamp with time zone,
    resolution_notification_sent_at timestamp with time zone,
    foreign key (organization_id, incident_id) references incidents (organization_id, id)
);

create index on incidents_notifications (creation_notification_sent_at);

-- automatically create an entry in this table every time a new incident is opened
create or replace function create_new_incident_notifications() returns trigger as $$
BEGIN
    INSERT INTO incidents_notifications (organization_id, incident_id) VALUES(new.organization_id, new.id);
END;
$$ language plpgsql;

create or replace trigger incidents_notifications_creation after insert on incidents execute procedure create_new_incident_notifications();