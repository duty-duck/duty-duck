-- Add down migration script here
drop index unique_ongoing_or_to_be_confirmed_incidents;
drop index unique_incident_events;