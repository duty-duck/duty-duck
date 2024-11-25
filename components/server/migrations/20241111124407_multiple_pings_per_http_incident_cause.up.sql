-- Add up migration script here
UPDATE incidents
SET cause = jsonb_build_object(
    'causeType', 'HttpMonitorIncidentCause',
    'lastPing', jsonb_build_object(
        'httpCode', (cause->>'httpCode')::integer,
        'errorKind', cause->>'errorKind'
    ),
    'previousPings', '[]'::jsonb
)
WHERE cause IS NOT NULL 
AND cause->>'causeType' = 'HttpMonitorIncidentCause';