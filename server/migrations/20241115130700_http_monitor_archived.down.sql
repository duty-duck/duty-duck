-- Add down migration script here
alter table http_monitors drop column archived_at;
