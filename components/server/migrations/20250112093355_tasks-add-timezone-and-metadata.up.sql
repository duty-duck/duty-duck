-- Add up migration script here
ALTER TABLE tasks ADD COLUMN schedule_timezone text DEFAULT 'Etc/UTC';
ALTER TABLE tasks ADD COLUMN metadata JSONB;
ALTER TABLE task_runs ADD COLUMN metadata JSONB;

CREATE INDEX ON tasks USING GIN (metadata);
CREATE INDEX ON task_runs USING GIN (metadata);


