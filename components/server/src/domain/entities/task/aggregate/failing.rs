use super::*;

/// These are the only states a task run can be in for the related task to be failing
pub enum FailingTaskRun {
    Failed(FailedTaskRun),
    Dead(DeadTaskRun),
}

pub struct FailingTaskAggregate {
    pub(super) task: FailingTask,
    pub(super) task_run: FailingTaskRun,
}


