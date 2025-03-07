use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::Serialize;
use serde_json::Value;

use crate::{ClientResult, DutyDuckApiClient, ResponseExtention};

/// A request to store the logs associated with a running task so they can be consulted and searched.
#[derive(Debug, Serialize, Default)]
pub struct SendTaskLogsRequest {
    pub events: Vec<TaskLogEvent>,
}

/// A single log event from a task run. (By default, each new line in the standard output is considered a seperate event)
#[derive(Debug, Serialize)]
pub struct TaskLogEvent {
    /// severity text (also known as log level). This is the original string representation of the severity as it is known at the source, for instance "DEBUG", or "ERROR".
    pub severity_text: Option<String>,
    /// SeverityNumber is an integer number. Smaller numerical values correspond to less severe events (such as debug events), larger numerical values correspond to more severe events (such as errors and critical events).
    /// The meaning of this value is defined by the OpenTelemetry standard.
    pub severity_number: Option<i32>,
    /// The body of the log. This can be unstructured data (i.e. a string of text) or a structured map of keys and values
    /// If structured data is desired, the client is responsible for supplying the structured data.
    pub body: Value,
    /// The timestamp at which the log occured, as measured by the orgin clock
    pub timestamp: DateTime<Utc>,
}

/// A builder to build and send requests to start a task
pub struct StartTaskBuilder {
    pub(super) new_task: Option<NewTask>,
    pub(super) task_id: String,
    pub(super) client: DutyDuckApiClient,
    pub(super) abort_previous_running_task: bool,
}

impl StartTaskBuilder {
    pub fn with_new_task(mut self, new_task: NewTask) -> Self {
        self.new_task = Some(new_task);
        self
    }

    pub fn abort_previous_running_task(mut self) -> Self {
        self.abort_previous_running_task = true;
        self
    }

    pub async fn send(self) -> ClientResult<()> {
        let url = self
            .client
            .base_url
            .join(&format!("/tasks/{}/start", self.task_id))
            .unwrap();
        let command = StartTaskCommand {
            new_task: self.new_task,
            abort_previous_running_task: self.abort_previous_running_task,
        };
        self.client
            .request(Method::POST, url)?
            .json(&command)
            .send()
            .await?
            .ok_or_err()
            .await
    }
}

/// A builder to build and send requests to finish a task
pub struct FinishTaskBuilder {
    pub(super) task_id: String,
    pub(super) client: DutyDuckApiClient,
    pub(super) status: FinishedTaskStatus,
    pub(super) exit_code: Option<i32>,
    pub(super) error_message: Option<String>,
}

impl FinishTaskBuilder {
    pub fn failure(mut self) -> Self {
        self.status = FinishedTaskStatus::Failure;
        self
    }

    pub fn aborted(mut self) -> Self {
        self.status = FinishedTaskStatus::Aborted;
        self
    }

    pub fn success(mut self) -> Self {
        self.status = FinishedTaskStatus::Success;
        self
    }

    pub fn with_exit_code(mut self, exit_code: i32) -> Self {
        self.exit_code = Some(exit_code);
        self
    }

    pub fn with_error_message(mut self, error_message: impl Into<String>) -> Self {
        self.error_message = Some(error_message.into());
        self
    }

    pub async fn send(self) -> ClientResult<()> {
        let url = self
            .client
            .base_url
            .join(&format!("/tasks/{}/finish", self.task_id))
            .unwrap();
        let command = FinishTaskCommand {
            status: self.status,
            exit_code: self.exit_code,
            error_message: self.error_message,
        };
        self.client
            .request(Method::POST, url)?
            .json(&command)
            .send()
            .await?
            .ok_or_err()
            .await
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskCommand {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub cron_schedule: Option<String>,
    pub start_window_seconds: Option<u32>,
    pub lateness_window_seconds: Option<u32>,
    pub heartbeat_timeout_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StartTaskCommand {
    new_task: Option<NewTask>,
    abort_previous_running_task: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTask {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cron_schedule: Option<String>,
    pub start_window_seconds: Option<u32>,
    pub lateness_window_seconds: Option<u32>,
    pub heartbeat_timeout_seconds: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum FinishedTaskStatus {
    Success,
    Failure,
    Aborted,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FinishTaskCommand {
    status: FinishedTaskStatus,
    exit_code: Option<i32>,
    error_message: Option<String>,
}
