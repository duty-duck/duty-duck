-- Add up migration script here

-- Add columns to the tasks table to control which notifications are enabled
alter table tasks
    add column email_notification_enabled boolean not null default true,
    add column push_notification_enabled boolean not null default true,
    add column sms_notification_enabled boolean not null default false;

-- Change the type of the id column to uuid and store the user-provided id in the user_id column

-- Create a new id coumn with type uuid and make it the new primary key of the table
alter table tasks
    drop constraint if exists tasks_id_check cascade,
    drop constraint if exists tasks_pkey cascade;

-- drop the foreign key constraint on the task_runs table
alter table task_runs drop constraint if exists task_runs_organization_id_task_id_fkey;

alter table tasks
    rename column id to user_id;

alter table tasks
    add column id uuid not null default gen_random_uuid(),
    add primary key (organization_id, id);

-- Add a unique constraint on the user_id and organization_id columns
alter table tasks
    add constraint tasks_user_id_organization_id_unique unique (user_id, organization_id);

-- Rename the task_id column to user_id
alter table task_runs
    rename column task_id to task_user_id;

-- Add a task_id column with type uuid on task runs
alter table task_runs add column task_id uuid;

-- Populate the task_id column of the task_runs table with the uuids of the task that correspond to the old user_ids
update task_runs
    set task_id = tasks.id
    from tasks
    where task_runs.task_user_id = tasks.user_id;

-- Make the task_id column not null
alter table task_runs alter column task_id set not null;

-- Add a foreign key constraint on the task_id column
alter table task_runs
    add constraint task_runs_organization_id_task_id_fkey foreign key (organization_id, task_id) references tasks (organization_id, id);
