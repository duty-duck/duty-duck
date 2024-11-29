-- Add up migration script here

CREATE TABLE tasks (
    id TEXT NOT NULL CHECK (id <> '' AND id !~ '\s'),
    organization_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status smallint not null,
    previous_status smallint,
    last_status_change_at TIMESTAMPTZ,
    next_due_at TIMESTAMPTZ, -- the time at which the task is next expected to run. Null for non-cron and disabled tasks
    cron_schedule text, -- NULL for non-cron tasks
    start_window_seconds INTEGER NOT NULL, -- Time before task is considered late
    lateness_window_seconds INTEGER NOT NULL, -- Time after task is considered late and before it is considered absent
    heartbeat_timeout_seconds INTEGER NOT NULL, -- Time after which task is considered dead without heartbeat
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (organization_id, id)
);

CREATE TABLE task_runs (
    organization_id UUID NOT NULL,
    task_id TEXT NOT NULL,
    status smallint NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    exit_code INTEGER, -- NULL if still running or dead or aborted
    error_message TEXT,
    last_heartbeat_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (
        organization_id,
        task_id,
        started_at
    ),
    FOREIGN KEY (organization_id, task_id) REFERENCES tasks (organization_id, id) on delete cascade
)
PARTITION BY
    RANGE (started_at);

-- used to store events related to a single task run
CREATE TABLE task_run_events (
    organization_id UUID NOT NULL,
    task_id TEXT NOT NULL,
    task_run_started_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    event_type smallint NOT NULL,
    event_payload JSONB,
    PRIMARY KEY (
        organization_id,
        task_id,
        task_run_started_at,
        created_at
    ),
    FOREIGN KEY (
        organization_id,
        task_id,
        task_run_started_at
    ) REFERENCES task_runs (
        organization_id,
        task_id,
        started_at
    ) on delete cascade,
    FOREIGN KEY (organization_id, task_id) REFERENCES tasks (organization_id, id) on delete cascade
)
PARTITION BY
    RANGE (created_at);

-- Function to create future partitions automatically
CREATE OR REPLACE FUNCTION create_task_runs_partition_for_month()
RETURNS void AS $$
DECLARE
    next_month_start date;
    next_month_end date;
    partition_name text;
BEGIN
    next_month_start := date_trunc('month', now()) + interval '1 month';
    next_month_end := next_month_start + interval '1 month';
    partition_name := 'task_runs_y' || 
                     to_char(next_month_start, 'YYYY') ||
                     'm' || to_char(next_month_start, 'MM');
    
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF task_runs
         FOR VALUES FROM (%L) TO (%L)',
        partition_name,
        next_month_start,
        next_month_end
    );
END;
$$ LANGUAGE plpgsql;

-- Function to create future partitions automatically
CREATE OR REPLACE FUNCTION create_task_run_events_partition_for_month()
RETURNS void AS $$
DECLARE
    next_month_start date;
    next_month_end date;
    partition_name text;
BEGIN
    next_month_start := date_trunc('month', now()) + interval '1 month';
    next_month_end := next_month_start + interval '1 month';
    partition_name := 'task_run_events_y' || 
                     to_char(next_month_start, 'YYYY') ||
                     'm' || to_char(next_month_start, 'MM');
    
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF task_run_events
         FOR VALUES FROM (%L) TO (%L)',
        partition_name,
        next_month_start,
        next_month_end
    );
END;
$$ LANGUAGE plpgsql;

-- create the first partition
SELECT create_task_runs_partition_for_month ();

SELECT create_task_run_events_partition_for_month ();