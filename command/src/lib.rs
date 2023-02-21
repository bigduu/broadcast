#![allow(dead_code)]
use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tracing::error;
use utils::snowflake::SNOWFLAKE;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub command: String,
    pub args: Vec<String>,
}

impl Command {
    fn new(command: String, args: Vec<String>) -> Self {
        Command {
            id: SNOWFLAKE.lock().unwrap().generate().to_string(),
            command,
            args,
        }
    }

    pub fn new_kill_command() -> Self {
        Command::kill_command()
    }

    pub fn new_open_command() -> Self {
        Command::open_command()
    }

    pub fn execute_command(&self) -> JoinHandle<CommandResult> {
        let cloned = self.clone();
        tokio::task::spawn_blocking(move || {
            match std::process::Command::new(cloned.command.clone())
                .args(cloned.args.clone())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
            {
                Ok(mut output) => match output.wait() {
                    Ok(status) => CommandResult::new(cloned.id, status.to_string()),
                    Err(e) => {
                        error!("Failed to wait command: {:?}, with error: {:?}", cloned, e);
                        CommandResult::new(
                            cloned.id,
                            format!("Failed to wait command with error: {e}"),
                        )
                    }
                },
                Err(e) => {
                    error!(
                        "Failed to execute command: {:?}, with error: {:?}",
                        cloned, e
                    );
                    CommandResult::new(cloned.id, format!("Failed to wait command with error: {e}"))
                }
            }
        })
    }
}

impl Command {
    #[cfg(target_os = "linux")]
    fn kill_command() -> Self {
        Command::new(
            "pkill".to_string(),
            vec!["-f".to_string(), "broadcast".to_string()],
        )
    }

    #[cfg(target_os = "windows")]
    fn kill_command() -> Self {
        Command::new(
            "taskkill".to_string(),
            vec![
                "/f".to_string(),
                "/im".to_string(),
                "broadcast.exe".to_string(),
            ],
        )
    }

    #[cfg(target_os = "macos")]
    fn kill_command() -> Self {
        Command::new(
            "pkill".to_string(),
            vec!["-f".to_string(), "broadcast".to_string()],
        )
    }

    #[cfg(target_os = "linux")]
    fn open_command() -> Self {
        Command::new("broadcast".to_string(), vec![])
    }

    #[cfg(target_os = "windows")]
    fn open_command() -> Self {
        Command::new("broadcast.exe".to_string(), vec![])
    }

    #[cfg(target_os = "macos")]
    fn open_command() -> Self {
        Command::new("broadcast".to_string(), vec![])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandResult {
    pub id: String,
    pub status: String,
}

impl CommandResult {
    pub fn new(id: String, status: String) -> Self {
        CommandResult { id, status }
    }
}
