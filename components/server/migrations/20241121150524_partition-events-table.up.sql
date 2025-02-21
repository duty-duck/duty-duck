-- 1. Rename existing table
DROP TABLE IF EXISTS incident_timeline_events_old;
ALTER TABLE incident_timeline_events
    RENAME TO incident_timeline_events_old;
-- 2. Create the new partitioned table
CREATE TABLE incident_timeline_events (
    organization_id uuid NOT NULL,
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    incident_id uuid NOT NULL,
    user_id uuid,
    created_at timestamp with time zone NOT NULL,
    event_type smallint NOT NULL,
    event_payload jsonb
) PARTITION BY RANGE (created_at);
-- Add an index to retrieve events for a given incident efficiently
CREATE INDEX incident_timeline_events_incident_idx ON incident_timeline_events(organization_id, incident_id);
-- Function to create future partitions automatically
CREATE OR REPLACE FUNCTION create_incident_timeline_partition_for_month() RETURNS void AS $$
DECLARE current_month_start date;
current_month_end date;
next_month_start date;
next_month_end date;
current_partition_name text;
next_partition_name text;
BEGIN current_month_start := date_trunc('month', now());
current_month_end := current_month_start + interval '1 month';
next_month_start := current_month_end;
next_month_end := next_month_start + interval '1 month';
current_partition_name := 'incident_timeline_events_y' || to_char(current_month_start, 'YYYY') || 'm' || to_char(current_month_start, 'MM');
next_partition_name := 'incident_timeline_events_y' || to_char(next_month_start, 'YYYY') || 'm' || to_char(next_month_start, 'MM');
EXECUTE format(
    'CREATE TABLE IF NOT EXISTS %I PARTITION OF incident_timeline_events
         FOR VALUES FROM (%L) TO (%L)',
    current_partition_name,
    current_month_start,
    current_month_end
);
EXECUTE format(
    'CREATE TABLE IF NOT EXISTS %I PARTITION OF incident_timeline_events
         FOR VALUES FROM (%L) TO (%L)',
    next_partition_name,
    next_month_start,
    next_month_end
);
END;
$$ LANGUAGE plpgsql;