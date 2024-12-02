-- 1. Rename existing table
-- (drop the old table if it exists to alllow this migration to be reverted and re-run)
DROP TABLE IF EXISTS incident_timeline_events_old;

ALTER TABLE incident_timeline_events
RENAME TO incident_timeline_events_old;

-- 2. Create the new partitioned table
CREATE TABLE incident_timeline_events (
    organization_id uuid NOT NULL,
    incident_id uuid NOT NULL,
    user_id uuid,
    created_at timestamp with time zone NOT NULL,
    event_type smallint NOT NULL,
    event_payload jsonb,
    PRIMARY KEY (
        organization_id,
        incident_id,
        created_at
    )
)
PARTITION BY
    RANGE (created_at);

-- 3. Creating partitions from September 2024 to November 2025
CREATE TABLE IF NOT EXISTS incident_timeline_events_y2024m09 PARTITION OF incident_timeline_events FOR
VALUES
FROM ('2024-09-01') TO ('2024-10-01');

CREATE TABLE IF NOT EXISTS incident_timeline_events_y2024m10 PARTITION OF incident_timeline_events FOR
VALUES
FROM ('2024-10-01') TO ('2024-11-01');

CREATE TABLE IF NOT EXISTS incident_timeline_events_y2024m11 PARTITION OF incident_timeline_events FOR
VALUES
FROM ('2024-11-01') TO ('2024-12-01');

CREATE TABLE IF NOT EXISTS incident_timeline_events_y2024m12 PARTITION OF incident_timeline_events FOR
VALUES
FROM ('2024-12-01') TO ('2025-01-01');
-- Create indices on each partition
CREATE UNIQUE INDEX IF NOT EXISTS incident_timeline_events_y202_organization_id_incident_id_e_idx ON incident_timeline_events_y2024m09 (
    organization_id,
    incident_id,
    event_type
)
where (
        event_type = 0
        or event_type = 2
        or event_type = 5
    );

CREATE UNIQUE INDEX IF NOT EXISTS incident_timeline_events_y202_organization_id_incident_id_e_idx1 ON incident_timeline_events_y2024m10 (
    organization_id,
    incident_id,
    event_type
)
where (
        event_type = 0
        or event_type = 2
        or event_type = 5
    );

CREATE UNIQUE INDEX IF NOT EXISTS incident_timeline_events_y202_organization_id_incident_id_e_idx2 ON incident_timeline_events_y2024m11 (
    organization_id,
    incident_id,
    event_type
)
where (
        event_type = 0
        or event_type = 2
        or event_type = 5
    );

CREATE UNIQUE INDEX IF NOT EXISTS incident_timeline_events_y202_organization_id_incident_id_e_idx3 ON incident_timeline_events_y2024m12 (
    organization_id,
    incident_id,
    event_type
)
where (
        event_type = 0
        or event_type = 2
        or event_type = 5
    );

-- Function to create future partitions automatically
CREATE OR REPLACE FUNCTION create_incident_timeline_partition_for_month()
RETURNS void AS $$
DECLARE
    next_month_start date;
    next_month_end date;
    partition_name text;
BEGIN
    next_month_start := date_trunc('month', now()) + interval '1 month';
    next_month_end := next_month_start + interval '1 month';
    partition_name := 'incident_timeline_events_y' || 
                     to_char(next_month_start, 'YYYY') ||
                     'm' || to_char(next_month_start, 'MM');
    
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF incident_timeline_events
         FOR VALUES FROM (%L) TO (%L)',
        partition_name,
        next_month_start,
        next_month_end
    );
    
    EXECUTE format(
        'CREATE UNIQUE INDEX IF NOT EXISTS %I ON %I (organization_id, incident_id, event_type) where (event_type = 0 or event_type = 2 or event_type = 5)',
        'idx_' || partition_name || '_unique_event',
        partition_name
    );
END;
$$ LANGUAGE plpgsql;