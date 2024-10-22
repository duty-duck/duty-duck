-- Add up migration script here
ALTER TABLE incidents
    ADD COLUMN IF NOT EXISTS incident_source_type smallint,
    ADD COLUMN IF NOT EXISTS incident_source_id uuid;
    
update incidents set incident_source_type = 0, incident_source_id = subquery.http_monitor_id
from (select * from http_monitors_incidents) as subquery
where subquery.organization_id = incidents.organization_id and subquery.incident_id = incidents.id;

alter table incidents 
	alter column incident_source_type set not null,
	alter column incident_source_id set not null;

-- Ensure that we can filter by incident source type and id efficiently.
create index on incidents (incident_source_type, incident_source_id);

drop table http_monitors_incidents;