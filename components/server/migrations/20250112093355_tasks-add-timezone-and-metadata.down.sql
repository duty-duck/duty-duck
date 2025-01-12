-- Drop the GIN index first
DROP INDEX IF EXISTS tasks_metadata_idx;

-- Remove the added columns
ALTER TABLE tasks DROP COLUMN IF EXISTS metadata;
ALTER TABLE tasks DROP COLUMN IF EXISTS schedule_timezone;
ALTER TABLE task_runs DROP COLUMN IF EXISTS metadata;
