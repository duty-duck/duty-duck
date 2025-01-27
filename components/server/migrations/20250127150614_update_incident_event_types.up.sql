-- Add up migration script here
UPDATE incident_timeline_events
SET event_type = 
    CASE event_type
        -- Monitor events (100-199 range)
        WHEN 6 THEN 100   -- MonitorPinged
        WHEN 7 THEN 101   -- MonitorSwitchedToRecovering
        WHEN 8 THEN 102   -- MonitorSwitchedToSuspicious
        WHEN 9 THEN 103   -- MonitorSwitchedToDown
        -- Task events (200-299 range)
        WHEN 10 THEN 200  -- TaskSwitchedToDue
        WHEN 11 THEN 201  -- TaskSwitchedToLate
        WHEN 12 THEN 202  -- TaskSwitchedToAbsent
        -- Task run events (300-399 range)
        WHEN 14 THEN 300  -- TaskRunStarted
        WHEN 15 THEN 301  -- TaskRunIsDead
        WHEN 16 THEN 302  -- TaskRunFailed
        ELSE event_type   -- Keep values 0-5 unchanged
    END;