-- Add up migration script here
CREATE TABLE tasks (
    id uuid not null default gen_random_uuid(),
    user_id TEXT NOT NULL UNIQUE CHECK (
        user_id <> ''
        AND user_id !~ '\s'
    ),
    organization_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status smallint not null,
    previous_status smallint,
    last_status_change_at TIMESTAMPTZ,
    next_due_at TIMESTAMPTZ,
    -- the time at which the task is next expected to run. Null for non-cron and disabled tasks
    cron_schedule text,
    -- NULL for non-cron tasks
    schedule_timezone text DEFAULT 'Etc/UTC',
    start_window_seconds INTEGER NOT NULL,
    -- Time before task is considered late
    lateness_window_seconds INTEGER NOT NULL,
    -- Time after task is considered late and before it is considered absent
    heartbeat_timeout_seconds INTEGER NOT NULL,
    -- Time after which task is considered dead without heartbeat
    -- notification settings for the task and its runs
    email_notification_enabled boolean not null default true,
    push_notification_enabled boolean not null default true,
    sms_notification_enabled boolean not null default false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB,
    PRIMARY KEY (organization_id, id)
);
CREATE TABLE task_runs (
    organization_id UUID NOT NULL,
    id uuid not null default gen_random_uuid(),
    task_id UUID NOT NULL,
    status smallint NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    exit_code INTEGER,
    -- NULL if still running or dead or aborted
    error_message TEXT,
    last_heartbeat_at TIMESTAMPTZ,
    heartbeat_timeout_seconds INTEGER NOT NULL,
    -- Time after which task is considered dead without heartbeat (inherited from task)
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB,
    FOREIGN KEY (organization_id, task_id) REFERENCES tasks (organization_id, id) on delete cascade
) PARTITION BY RANGE (started_at);

-- Add an index to retrieve a specific task run
-- Note that we cannot make it a unique index because the table is partitioned by the started_at column.
-- In practice through, since we use random UUIDs, we should not have any collisions.
CREATE INDEX task_runs_org_id_idx ON task_runs(organization_id, id);
-- Add an index to retrieve task runs for a given task efficiently
CREATE INDEX task_runs_org_task_idx ON task_runs(organization_id, task_id);


-- used to store events related to a single task run
CREATE TABLE task_run_events (
    organization_id UUID NOT NULL,
    task_run_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    event_type smallint NOT NULL,
    event_payload JSONB,
    -- Add a primary key on the organization_id, task_run_id, and created_at columns
    -- This means that we cannot have two events with the same organization_id, task_run_id, and created_at
    -- But we can have two events with the same organization_id, task_run_id, and different dates
    PRIMARY KEY (
        organization_id,
        task_run_id,
        created_at
    )
) PARTITION BY RANGE (created_at);

-- Add an index to retrieve task run events for a given task run efficiently
-- Note that sadly we cannot add a foreign key to the task_run_events table, becasue it would require a unique constraint
-- on task runs ids, which would make it impossible to partition the table.
CREATE INDEX task_run_events_org_task_run_idx ON task_run_events(organization_id, task_run_id);

-- Function to create future partitions automatically
CREATE OR REPLACE FUNCTION create_task_runs_partition_for_month()
RETURNS void AS $$
DECLARE
    current_month_start date;
    current_month_end date;
    next_month_start date;
    next_month_end date;
    current_partition_name text;
    next_partition_name text;
BEGIN
    current_month_start := date_trunc('month', now());
    current_month_end := current_month_start + interval '1 month';
    next_month_start := current_month_end;
    next_month_end := next_month_start + interval '1 month';

    current_partition_name := 'task_runs_y' || 
                     to_char(current_month_start, 'YYYY') ||
                     'm' || to_char(current_month_start, 'MM');
    
    next_partition_name := 'task_runs_y' || 
                     to_char(next_month_start, 'YYYY') ||
                     'm' || to_char(next_month_start, 'MM');
    
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF task_runs
         FOR VALUES FROM (%L) TO (%L)',
        current_partition_name,
        current_month_start,
        current_month_end
    );
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF task_runs
         FOR VALUES FROM (%L) TO (%L)',
        next_partition_name,
        next_month_start,
        next_month_end
    );
END;
$$ LANGUAGE plpgsql;
-- Function to create future partitions automatically
CREATE OR REPLACE FUNCTION create_task_run_events_partition_for_month()
RETURNS void AS $$
DECLARE
    current_month_start date;
    current_month_end date;
    next_month_start date;
    next_month_end date;
    current_partition_name text;
    next_partition_name text;
BEGIN
    current_month_start := date_trunc('month', now());
    current_month_end := current_month_start + interval '1 month';
    next_month_start := current_month_end;
    next_month_end := next_month_start + interval '1 month';

    current_partition_name := 'task_run_events_y' || 
                     to_char(current_month_start, 'YYYY') ||
                     'm' || to_char(current_month_start, 'MM');
    
    next_partition_name := 'task_run_events_y' || 
                     to_char(next_month_start, 'YYYY') ||
                     'm' || to_char(next_month_start, 'MM');
    
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF task_run_events
         FOR VALUES FROM (%L) TO (%L)',
        current_partition_name,
        current_month_start,
        current_month_end
    );
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF task_run_events
         FOR VALUES FROM (%L) TO (%L)',
        next_partition_name,
        next_month_start,
        next_month_end
    );
END;
$$ LANGUAGE plpgsql;
-- create the first partition
SELECT create_task_runs_partition_for_month ();
SELECT create_task_run_events_partition_for_month ();