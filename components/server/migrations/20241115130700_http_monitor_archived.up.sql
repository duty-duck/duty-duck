-- Add up migration script here
alter table http_monitors add column archived_at timestamp with time zone;
