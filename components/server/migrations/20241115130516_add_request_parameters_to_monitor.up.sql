-- Add up migration script here
alter table http_monitors add column request_timeout_ms int not null default 10000;
alter table http_monitors add column request_headers jsonb not null default '{}';
