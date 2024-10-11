-- Add up migration script here
create table api_access_tokens (
    id uuid primary key default gen_random_uuid(),
    organization_id uuid not null references organizations(id),
    user_id uuid not null,
    label text not null,
    secret_key text not null unique,
    created_at timestamptz not null default now(),
    expires_at timestamptz not null,
    scopes smallint[] not null default '{}'::smallint[]
);
