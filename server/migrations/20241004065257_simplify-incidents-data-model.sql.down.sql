-- Add down migration script here
create table if not exists http_monitors_incidents (
    organization_id uuid not null,
    incident_id uuid not null,
    http_monitor_id uuid not null,
    primary key (organization_id, http_monitor_id, incident_id),
    foreign key (organization_id, incident_id) references incidents (organization_id, id) on delete cascade,
    foreign key (organization_id, http_monitor_id) references http_monitors (organization_id, id) on delete cascade
);

insert into http_monitors_incidents (organization_id, incident_id, http_monitor_id)
select organization_id, id, incident_source_id from incidents where incidents.incident_source_type = 0
on conflict do nothing;

alter table incidents drop column incident_source_type, drop column incident_source_id;