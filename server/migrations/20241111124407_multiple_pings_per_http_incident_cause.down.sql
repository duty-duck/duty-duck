-- Add down migration script here
UPDATE incidents
SET cause = jsonb_build_object(
    'causeType', 'HttpMonitorIncidentCause',
    'httpCode', (cause->'lastPing'->>'httpCode')::jsonb,
    'errorKind', cause->'lastPing'->>'errorKind'
)
WHERE cause IS NOT NULL 
AND cause->>'causeType' = 'HttpMonitorIncidentCause';