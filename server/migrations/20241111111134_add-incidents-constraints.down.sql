-- Add down migration script here
alter table incidents drop constraint incidents_status_check;
alter table incidents drop constraint incidents_resolved_at_status_check;
