-- Remove notification settings columns from tasks table
alter table tasks
    drop column email_notification_enabled,
    drop column push_notification_enabled,
    drop column sms_notification_enabled;

-- Drop the foreign key constraint on task_runs
alter table task_runs
    drop constraint task_runs_organization_id_task_id_fkey;

-- Drop the unique constraint on user_id and organization_id
alter table tasks
    drop constraint tasks_user_id_organization_id_unique;

-- Drop the primary key constraint
alter table tasks
    drop constraint tasks_pkey;

-- Remove the uuid column and restore the original id column
alter table task_runs
    drop column task_id;

alter table task_runs
    rename column task_user_id to task_id;

alter table tasks
    drop column id;

alter table tasks
    rename column user_id to id;

-- Restore the primary key on the original id column
alter table tasks
    add primary key (organization_id, id);

-- Restore the foreign key constraint on task_runs
alter table task_runs
    add constraint task_runs_organization_id_task_id_fkey 
    foreign key (organization_id, task_id) 
    references tasks (organization_id, id);
