use std::{process::Stdio, time::Duration};

use crate::config::Config;
use anyhow::Context;
use chrono::Utc;
use clap::*;
use dutyduck_api_client_rs::{
    ClientError, ClientResult, DutyDuckApiClient, NewTask, SendTaskLogsRequest, TaskRunLogEvent,
};
use futures::StreamExt;
use reqwest::StatusCode;
use serde_json::json;
use tokio::process::Child;
use tokio_util::codec::{FramedRead, LinesCodec};

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
    fn log_send_logs_result(result: ClientResult<()>) {
        if let Err(e) = result {
            tracing::warn!(error = ?e, "Failed to send logs to the platform");
        }
    }

    let client = client.tasks();

    let started_at = Utc::now();
    let command_string = format!("{} {}", command.command, command.args.join(" "));

    let mut process: Child = tokio::process::Command::new(&command.command)
        .args(&command.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
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
            name: command.name.or(Some(command.command.clone())),
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

    // launch a background task to capture the process output
    let capture_output_task = tokio::spawn({
        let client = client.clone();
        let task_id = command.task_id.clone();

        let stdout_events = FramedRead::new(process.stdout.take().unwrap(), LinesCodec::new())
            .filter_map(|line| async move { line.ok() })
            .map(|line| {
                // print the line to the standard output
                println!("{line}");
                log_line_to_event(line, LogLineSeverity::Info)
            });
        let stderr_events = FramedRead::new(process.stderr.take().unwrap(), LinesCodec::new())
            .filter_map(|line| async move { line.ok() })
            .map(|line| {
                // print the line to standard error
                eprintln!("{line}");
                log_line_to_event(line, LogLineSeverity::Error)
            });
        let mut merged_stream = futures::stream::select(stdout_events, stderr_events).boxed();

        async move {
            // Begin the lgos by an event describing the launched command
            let mut current_batch = SendTaskLogsRequest {
                events: vec![TaskRunLogEvent {
                    severity_text: Some("INFO".to_string()),
                    severity_number: Some(9),
                    body: json!({
                        "message": format!("[DutyDuck CLI] Starting task with command: {command_string}"),
                        "command": command.command,
                        "command_args": command.args,
                        "pwd": std::env::current_dir().ok()
                    }),
                    timestamp: started_at,
                }],
            };
            let mut send_current_batch_interval = tokio::time::interval(Duration::from_secs(2));

            loop {
                tokio::select! {
                    event = merged_stream.next() => {
                        match event {
                            Some(event) => {
                                current_batch.events.push(event);

                                if current_batch.events.len() >= 100 {
                                    log_send_logs_result(client.send_logs(&task_id, &std::mem::take(&mut current_batch)).await);
                                }
                            },
                            None => {
                                tracing::info!("Task finished");
                                if !current_batch.events.is_empty() {
                                    tracing::info!("Sending last log events");
                                    log_send_logs_result(client.send_logs(&task_id, &current_batch).await);
                                }
                                break;
                            }
                        }
                    }
                    _ = send_current_batch_interval.tick() => {
                        if !current_batch.events.is_empty() {
                            log_send_logs_result(client.send_logs(&task_id, &std::mem::take(&mut current_batch)).await);
                        }
                    }
                };
            }
        }
    });

    // launch a background task to send periodic heartbeats
    let send_heartbeat_task = tokio::spawn({
        let client = client.clone();
        let task_id = command.task_id.clone();

        async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
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

    // Wait for the child to finish. The child can either finish on its own, or we can kill it
    // when a CTRL+C signal is received.
    let ctrl_c = tokio::signal::ctrl_c();
    let finish_task_request = tokio::select! {
        child_exit = process.wait() => {
            match child_exit {
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
            }
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

        }
    };

    // wait for the logs task to terminate
    capture_output_task.await?;

    // abort the send heartbeat task and wait of it to finish
    send_heartbeat_task.abort();
    let _ = send_heartbeat_task.await;

    // send a request to mark the task as fnished
    // this has to be done after the last logs have been sent, because we can't send logs once the task has finished
    finish_task_request.send().await?;

    tracing::info!("Done!");

    Ok(())
}

enum LogLineSeverity {
    Info,
    Error,
}

// todo: enhance parsing to allow users to exatract structured data from their logs
fn log_line_to_event(line: String, severity: LogLineSeverity) -> TaskRunLogEvent {
    let (sevrity_text, severity_number) = match severity {
        LogLineSeverity::Error => ("ERROR".to_string(), 17),
        LogLineSeverity::Info => ("INFO".to_string(), 9),
    };
    TaskRunLogEvent {
        timestamp: Utc::now(),
        severity_number: Some(severity_number),
        severity_text: Some(sevrity_text),
        body: serde_json::Value::String(line),
    }
}
