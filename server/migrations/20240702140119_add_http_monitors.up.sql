-- Add up migration script here
create table if not exists http_monitors (
   organization_id uuid not null,
   id uuid not null default gen_random_uuid(),
   created_at timestamp with time zone not null default now(),
   url text not null,
   status smallint not null,
   first_ping_at timestamp with time zone,
   next_ping_at timestamp with time zone,
   last_ping_at timestamp with time zone,
   interval_seconds int not null,
   last_http_code smallint,
   constraint http_monitors_pkey primary key (organization_id, id)
);

create index if not exists http_monitor_organization_id_idx on http_monitors (organization_id);
create index if not exists http_monitor_next_ping_at_idx on http_monitors (next_ping_at);