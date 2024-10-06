-- Add up migration script here
create table if not exists incident_timeline_events (
    organization_id uuid not null,
    incident_id uuid not null,
    created_at timestamptz not null,
    -- 0: creation, 1: notification, 2: resolution, 3: comment
    event_type smallint not null,
    event_payload jsonb,
    primary key (organization_id, incident_id, created_at),
    foreign key (organization_id, incident_id) references incidents (organization_id, id) on delete cascade
);

-- add a creation event for all incidents
insert into incident_timeline_events (
        organization_id,
        incident_id,
        created_at,
        event_type
    )
select organization_id,
    id,
    created_at,
    0
from incidents;
