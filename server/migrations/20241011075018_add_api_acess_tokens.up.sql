-- Add up migration script here
create table api_access_tokens (
    id uuid primary key default gen_random_uuid(),
    organization_id uuid not null,
    user_id uuid not null,
    label text not null,
    -- the hashed secret key
    secret_key bytea not null unique,
    created_at timestamptz not null default now(),
    expires_at timestamptz not null,
    -- an array of Permission values
    scopes smallint[] not null default '{}'::smallint[]
);
