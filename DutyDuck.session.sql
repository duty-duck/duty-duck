select * from incidents i
left join http_monitors_incidents hmi on i.organization_id = hmi.organization_id and hmi.incident_id = i.id
left join http_monitors hm on hm.organization_id = hmi.organization_id and hm.id = hmi.http_monitor_id;

delete from incidents;

delete from http_monitors_incidents;

delete from http_monitors;