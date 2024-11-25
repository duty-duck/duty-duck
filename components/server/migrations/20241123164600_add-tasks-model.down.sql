DROP FUNCTION IF EXISTS create_task_runs_partition_for_month();
DROP FUNCTION IF EXISTS create_task_run_events_partition_for_month();

-- Drop the tables in reverse order (due to foreign key dependencies)
DROP TABLE IF EXISTS task_run_events;
DROP TABLE IF EXISTS task_runs;
DROP TABLE IF EXISTS tasks;