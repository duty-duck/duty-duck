create table if not exists incidents (
    organization_id uuid not null,
    id uuid not null default gen_random_uuid(),
    created_at timestamp with time zone not null default now(),
    created_by uuid,
    resolved_at timestamp with time zone,
    cause jsonb,
    status smallint not null,
    priority smallint not null,
    primary key (organization_id, id)
);

create index on incidents (organization_id);

create table if not exists http_monitors_incidents (
    organization_id uuid not null,
    incident_id uuid not null,
    http_monitor_id uuid not null,
    primary key (organization_id, http_monitor_id, incident_id),
    foreign key (organization_id, incident_id) references incidents (organization_id, id) on delete cascade,
    foreign key (organization_id, http_monitor_id) references http_monitors (organization_id, id) on delete cascade
);