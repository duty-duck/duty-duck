use std::time::Duration;

use crate::config::Config;
use anyhow::Context;
use api_client_rs::{ClientError, DutyDuckApiClient, NewTask};
use clap::*;
use reqwest::StatusCode;
use tokio::process::Child;

#[derive(Subcommand)]
pub enum TasksCommands {
    /// Run a process locally, wrapped in a task run. The status of the process will be reported back to the platform.
    Run(RunCommand),
}

#[derive(Args)]
pub struct RunCommand {
    #[arg(long)]
    /// The id of the task to run
    pub task_id: String,
    /// Create the task if it does not exist
    #[arg(long)]
    pub create: bool,
    /// Whether to abort the previous running task if there is a running task with the same id
    #[arg(long)]
    pub abort_previous_running_task: bool,
    /// The name of the newly-created task
    #[arg(long)]
    pub name: Option<String>,
    /// The description of the newly-created task
    #[arg(long)]
    pub description: Option<String>,
    /// The cron schedule of the newly-created task
    #[arg(long)]
    pub cron_schedule: Option<String>,
    /// The start window of the newly-created task
    #[arg(long)]
    pub start_window_seconds: Option<u32>,
    /// The lateness window of the newly-created task
    #[arg(long)]
    pub lateness_window_seconds: Option<u32>,
    /// The heartbeat timeout of the newly-created task
    #[arg(long)]
    pub heartbeat_timeout_seconds: Option<u32>,
    /// The command to run
    pub command: String,
    /// The arguments to pass to the command
    pub args: Vec<String>,
}

pub async fn handle_tasks_command(command: TasksCommands) -> anyhow::Result<()> {
    let config = Config::load().await?;
    let client = config.get_api_client()?;

    match command {
        TasksCommands::Run(command) => run_task(&client, command).await,
    }
}

async fn run_task(client: &DutyDuckApiClient, command: RunCommand) -> anyhow::Result<()> {
    let client = client.tasks();
    let mut process: Child = tokio::process::Command::new(&command.command)
        .args(command.args)
        // kill the process if the child handle is dropped, which allows the task to stop
        // if the platform reports that the task has been aborted
        .kill_on_drop(true)
        .spawn()
        .context("Failed to start child process")?;

    let mut request = client.start_task(&command.task_id);
    if command.abort_previous_running_task {
        request = request.abort_previous_running_task();
    }
    if command.create {
        request = request.with_new_task(NewTask {
            name: command.name.or(Some(command.command)),
            description: command.description,
            cron_schedule: command.cron_schedule,
            start_window_seconds: command.start_window_seconds,
            lateness_window_seconds: command.lateness_window_seconds,
            heartbeat_timeout_seconds: command.heartbeat_timeout_seconds,
        });
    }

    request
        .send()
        .await
        .context("Failed to send start task request")?;

    let heartbeat_interval = Duration::from_secs(10);
    let send_heartbeat_task = tokio::spawn({
        let client = client.clone();
        let task_id = command.task_id.clone();
        async move {
            let mut interval = tokio::time::interval(heartbeat_interval);
            loop {
                interval.tick().await;
                match client.send_heartbeat(&task_id).await {
                    Ok(_) => (),
                    // If the platform reports that the task is no longer running (i.e. it has been aborted),
                    // we can stop sending heartbeats and we can kill the local process
                    Err(ClientError::InvalidStatusCode(StatusCode::BAD_REQUEST, _)) => {
                        eprintln!(
                            "Tried to send a heartbeat but the task is no longer running. Maybe it was aborted?"
                        );
                        break;
                    }
                    Err(e) => eprintln!("Failed to send heartbeat: {}", e),
                }
            }
        }
    });

    // Add ctrl+c signal handler
    let ctrl_c = tokio::signal::ctrl_c();

    tokio::select! {
        child_exit = process.wait() => {
            let finish_request = match child_exit {
                Ok(status) => {
                    let mut request = client.finish_task(&command.task_id);
                    if let Some(exit_code) = status.code() {
                        request = request.with_exit_code(exit_code);
                    }
                    if !status.success() {
                        request = request.failure();
                    }
                    request
                }
                Err(e) => {
                    eprintln!("Failed to wait for child process: {}", e);
                    client.finish_task(&command.task_id).failure()
                }
            };
        
            finish_request
                .send()
                .await
                .context("Failed to send finish task request")?;
        }
        _ = send_heartbeat_task => {
            // if the heartbeat task completes before the child process, it can only mean that
            // the task was aborted, so we can kill the local process
            eprintln!("Task was aborted, killing subprocess");
            process.start_kill().context("Failed to kill subprocess")?;
        }
        _ = ctrl_c => {
            eprintln!("Received interrupt signal, gracefully shutting down...");
            
            // Kill the subprocess
            process.start_kill().context("Failed to kill subprocess")?;
            
            // Wait for the process to actually terminate
            process.wait().await.context("Failed to wait for subprocess to terminate")?;
            
            // Send failure status to the API
            client.finish_task(&command.task_id)
                .aborted()
                .send()
                .await
                .context("Failed to send finish task request")?;
            
            eprintln!("Graceful shutdown complete");
        }
    };

    Ok(())
}
