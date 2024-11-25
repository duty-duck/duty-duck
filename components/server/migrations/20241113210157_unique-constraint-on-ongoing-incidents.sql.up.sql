-- Add up migration script here

-- add a unique index to make sure that there is only one ongoing or to be confirmed incident per organization per incident source
create unique index unique_ongoing_or_to_be_confirmed_incidents on incidents (organization_id, incident_source_id) where (status = 1 or status = 2);

-- add a unique index to make sure that there is only one incident event per organization per incident per event type, for these event types only
-- 0: creation, 2: resolution, 5: confirmation
-- an incident can only be created, resolved or confirmed once, thus there cannot be multiple events of these types
create unique index unique_incident_events on incident_timeline_events (organization_id, incident_id, event_type) where (event_type = 0 or event_type = 2 or event_type = 5);