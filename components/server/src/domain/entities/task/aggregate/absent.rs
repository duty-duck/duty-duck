use super::*;
use chrono::{DateTime, Utc};

pub struct AbsentTaskAggregate {
    pub(super) task: AbsentTask,
}


impl AbsentTaskAggregate {
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTaskAggregate, TaskAggregateError> {
        let task = self.task.start(now)?;
        let task_run = RunningTaskRun::new(
            *task.base().organization_id(),
            task.base().id().clone(),
            now,
            *task.base().heartbeat_timeout(),
        );
        Ok(RunningTaskAggregate { task, task_run })
    }
}
