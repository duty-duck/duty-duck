-- Add down migration script here
alter table http_monitors drop column request_timeout_ms;
alter table http_monitors drop column request_headers;
