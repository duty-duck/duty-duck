use super::{
    AbsentTaskAggregate, LateTask, RunningTaskAggregate,
    RunningTaskRun, TaskAggregateError, TaskStatus,
};
use chrono::{DateTime, Utc};

pub struct LateTaskAggregate {
    pub(super) task: LateTask,
}

impl LateTaskAggregate {
    /// State transition: Late -> Running
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTaskAggregate, TaskAggregateError> {
        let task = self.task.start(now)?;
        let task_run = RunningTaskRun::new(
            *task.base().organization_id(),
            task.base().id().clone(),
            now,
        );
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
}
