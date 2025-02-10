use super::*;
use chrono::{DateTime, Utc};

#[derive(getset::Getters, Debug)]
pub struct LateTaskAggregate {
    #[getset(get = "pub")]
    pub(super) task: LateTask,
}

impl LateTaskAggregate {
    pub fn user_id(&self) -> &TaskUserId {
        self.task.base().user_id()
    }

    pub fn task_base(&self) -> &TaskBase {
        self.task.base()
    }

    /// State transition: Late -> Running
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTaskAggregate, TaskAggregateError> {
        let task = self.task.start(now)?;
        let task_run = RunningTaskRun::new(task.base(), now);
        Ok(RunningTaskAggregate { task, task_run })
    }

    /// Checks whether the task should transition to the absent state
    /// (outside the lateness window)
    pub fn is_absent(&self, now: DateTime<Utc>) -> bool {
        self.task.is_absent(now)
    }

    /// State transition: Late -> Absent
    pub fn mark_absent(
        self,
        now: DateTime<Utc>,
    ) -> Result<AbsentTaskAggregate, TaskAggregateError> {
        if !self.is_absent(now) {
            return Err(TaskAggregateError::InvalidStateTransition {
                from: (TaskStatus::Late, None),
                to: (TaskStatus::Absent, None),
                details: "this task is not absent".to_string(),
            });
        }
        Ok(AbsentTaskAggregate {
            task: self.task.mark_absent(now)?,
        })
    }

    /// State transition to Archived
    pub fn archive(self, now: DateTime<Utc>) -> ArchivedTaskAggregate {
        ArchivedTaskAggregate {
            task: self.task.archive(now),
        }
    }
}
