use std::str::FromStr;

use crate::domain::use_cases::tasks::CreateTaskCommand;

use super::*;

pub const DEFAULT_START_WINDOW: Duration = Duration::from_secs(120);
pub const DEFAULT_LATENESS_WINDOW: Duration = Duration::from_secs(240);
pub const DEFAULT_HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(30);

/// A task that is in a healthy state (not failed, not late, not failing)
pub struct HealthyTask {
    pub(super) base: TaskBase,
    pub(super) next_due_at: Option<DateTime<Utc>>,
}

impl HealthyTask {
    /// Create a new healthy task
    /// New tasks are always created with a status of Healthy
    pub fn new(organization_id: Uuid, command: CreateTaskCommand) -> Result<Self, TaskError> {
        let now = Utc::now();
        let cron_schedule = command
            .cron_schedule
            .map(|s| croner::Cron::from_str(&s).map_err(|_| TaskError::InvalidCronSchedule))
            .transpose()?;
        Ok(Self {
            next_due_at: calculate_next_due_at(&cron_schedule, now)?,
            base: TaskBase {
                name: command.name.unwrap_or_else(|| command.id.to_string()),
                id: command.id,
                organization_id,
                description: command.description,
                cron_schedule,
                start_window: command
                    .start_window_seconds
                    .map_or(DEFAULT_START_WINDOW, |secs| {
                        Duration::from_secs(secs.clamp(0, 3600) as u64)
                    }),
                lateness_window: command
                    .lateness_window_seconds
                    .map_or(DEFAULT_LATENESS_WINDOW, |secs| {
                        Duration::from_secs(secs.clamp(0, 3600) as u64)
                    }),
                heartbeat_timeout: command
                    .heartbeat_timeout_seconds
                    .map_or(DEFAULT_HEARTBEAT_TIMEOUT, |secs| {
                        Duration::from_secs(secs.clamp(10, 3600) as u64)
                    }),
                created_at: now,
                previous_status: None,
                last_status_change_at: Some(now),
            },
        })
    }

    /// State transition: Healthy -> Running
    pub fn start(self, now: DateTime<Utc>) -> Result<RunningTask, TaskError> {
        Ok(RunningTask {
            // When a task starts, its next_due_at field is updated to the next time the task is due to run
            next_due_at: calculate_next_due_at(&self.base.cron_schedule, now)?,
            base: TaskBase {
                previous_status: Some(TaskStatus::Healthy),
                last_status_change_at: Some(now),
                ..self.base
            },
        })
    }
}

impl TryFrom<HealthyTask> for BoundaryTask {
    type Error = TaskError;

    fn try_from(task: HealthyTask) -> Result<Self, Self::Error> {
        Ok(BoundaryTask {
            status: TaskStatus::Absent,
            next_due_at: task.next_due_at,
            ..BoundaryTask::from(task.base)
        })
    }
}

impl TryFrom<BoundaryTask> for HealthyTask {
    type Error = TaskError;

    fn try_from(boundary: BoundaryTask) -> Result<Self, Self::Error> {
        if boundary.status != TaskStatus::Healthy {
            return Err(TaskError::FailedToBuildFromBoundary {
                details: "task status must be healthy".to_string(),
            });
        }
        Ok(HealthyTask {
            next_due_at: boundary.next_due_at,
            base: boundary.try_into()?,
        })
    }
}
