-- Add down migration script here
ALTER TABLE http_monitors DROP COLUMN metadata;
ALTER TABLE incidents DROP COLUMN metadata;