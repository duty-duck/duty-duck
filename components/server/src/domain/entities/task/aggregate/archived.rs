use super::*;

#[derive(Debug, Clone, getset::Getters)]
#[getset(get = "pub")]
pub struct ArchivedTaskAggregate {
    pub(super) task: ArchivedTask,
}
