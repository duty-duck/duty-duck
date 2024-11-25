-- Add up migration script here
alter table incidents add constraint incidents_status_check check (status in (0, 1, 2));

-- Add constraints for resolved_at based on status
alter table incidents add constraint incidents_resolved_at_status_check check (
    -- resolved incidents must have a resolved_at date
    (status = 0 and resolved_at is not null) or
    -- ongoing and to be confirmed incidents must not have a resolved_at date
    (status in (1, 2) and resolved_at is null)
);

