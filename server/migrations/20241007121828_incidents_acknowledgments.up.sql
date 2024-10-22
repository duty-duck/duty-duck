-- Add up migration script here
alter table incidents add column acknowledged_by uuid[] not null default '{}';