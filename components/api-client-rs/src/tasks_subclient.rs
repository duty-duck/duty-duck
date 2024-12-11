use reqwest::Method;
use serde::Serialize;

use crate::{ClientResult, DutyDuckApiClient, ResponseExtention};

#[derive(Clone)]
pub struct TasksSubclient {
    pub(crate) client: DutyDuckApiClient,
}

impl TasksSubclient {
    pub async fn create_task(&self, command: CreateTaskCommand) -> ClientResult<()> {
        let url = self.client.base_url.join("/tasks").unwrap();
        self.client
            .request(Method::POST, url)?
            .json(&command)
            .send()
            .await?
            .ok_or_err()
            .await
    }

    pub fn start_task(&self, task_id: impl Into<String>) -> StartTaskBuilder {
        StartTaskBuilder {
            client: self.client.clone(),
            task_id: task_id.into(),
            new_task: None,
            abort_previous_running_task: false,
        }
    }

    pub async fn send_heartbeat(&self, task_id: &str) -> ClientResult<()> {
        let url = self
            .client
            .base_url
            .join(&format!("/tasks/{task_id}/heartbeat"))
            .unwrap();
        self.client
            .request(Method::POST, url)?
            .send()
            .await?
            .ok_or_err()
            .await
    }

    pub fn finish_task(&self, task_id: impl Into<String>) -> FinishTaskBuilder {
        FinishTaskBuilder {
            task_id: task_id.into(),
            client: self.client.clone(),
            status: FinishedTaskStatus::Success,
            exit_code: None,
            error_message: None,
        }
    }
}

pub struct StartTaskBuilder {
    new_task: Option<NewTask>,
    task_id: String,
    client: DutyDuckApiClient,
    abort_previous_running_task: bool,
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

pub struct FinishTaskBuilder {
    task_id: String,
    client: DutyDuckApiClient,
    status: FinishedTaskStatus,
    exit_code: Option<i32>,
    error_message: Option<String>,
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
enum FinishedTaskStatus {
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
