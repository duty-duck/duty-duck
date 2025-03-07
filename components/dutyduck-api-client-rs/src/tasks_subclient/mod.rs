use reqwest::Method;

mod types;

pub use types::*;

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

    pub fn start_task(&self, task_id: impl ToString) -> StartTaskBuilder {
        StartTaskBuilder {
            client: self.client.clone(),
            task_id: task_id.to_string(),
            new_task: None,
            abort_previous_running_task: false,
        }
    }

    pub async fn send_heartbeat(&self, task_id: impl ToString) -> ClientResult<()> {
        let url = self
            .client
            .base_url
            .join(&format!("/tasks/{}/heartbeat", task_id.to_string()))
            .unwrap();
        self.client
            .request(Method::POST, url)?
            .send()
            .await?
            .ok_or_err()
            .await
    }

    pub async fn send_logs(
        &self,
        task_id: impl ToString,
        request: &SendTaskLogsRequest,
    ) -> ClientResult<()> {
        let url = self
            .client
            .base_url
            .join(&format!("/tasks/{}/logs", task_id.to_string()))
            .unwrap();
        self.client
            .request(Method::POST, url)?
            .json(request)
            .send()
            .await?
            .ok_or_err()
            .await
    }

    pub fn finish_task(&self, task_id: impl ToString) -> FinishTaskBuilder {
        FinishTaskBuilder {
            task_id: task_id.to_string(),
            client: self.client.clone(),
            status: FinishedTaskStatus::Success,
            exit_code: None,
            error_message: None,
        }
    }
}
