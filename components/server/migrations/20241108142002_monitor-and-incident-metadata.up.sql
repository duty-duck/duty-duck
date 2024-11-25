-- Add up migration script here
ALTER TABLE http_monitors ADD COLUMN metadata JSONB;
ALTER TABLE incidents ADD COLUMN metadata JSONB;

CREATE INDEX ON http_monitors USING GIN (metadata);
CREATE INDEX ON incidents USING GIN (metadata);

ALTER TABLE http_monitors DROP COLUMN tags;