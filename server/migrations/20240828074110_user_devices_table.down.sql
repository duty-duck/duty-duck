-- Add down migration script here
drop table user_devices;
drop table incidents_notifications;
drop trigger incidents_notifications_creation on incidents;
drop function create_new_incident_notifications;