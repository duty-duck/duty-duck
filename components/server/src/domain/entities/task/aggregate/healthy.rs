use super::*;

/// These are the only states a task run can be in for the related task to be healthy
pub enum HealthyTaskRun {
    Aborted(AbortedTaskRun),
    Finished(FinishedTaskRun),
}

pub struct HealthyTaskAggregate {
    pub(super) task: HealthyTask,
    pub(super) last_task_run: Option<HealthyTaskRun>,
}


