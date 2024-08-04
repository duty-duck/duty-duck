select * from incidents i
left join http_monitors_incidents hmi on i.organization_id = hmi.organization_id and hmi.incident_id = i.id;

update incidents set status = 1;