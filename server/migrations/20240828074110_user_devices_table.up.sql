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