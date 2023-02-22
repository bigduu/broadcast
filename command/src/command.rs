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
    pub fn kill_command(process_name: &str) -> Self {
        Command::new(
            "pkill".to_string(),
            vec!["-f".to_string(), process_name.to_string()],
        )
    }

    #[cfg(target_os = "windows")]
    pub fn kill_command(process_name: &str) -> Self {
        let process_name = format!("{}{}", process_name, ".exe");
        Command::new(
            "taskkill".to_string(),
            vec!["/f".to_string(), "/im".to_string(), process_name],
        )
    }

    #[cfg(target_os = "macos")]
    pub fn kill_command(process_name: &str) -> Self {
        Command::new(
            "pkill".to_string(),
            vec!["-f".to_string(), process_name.to_string()],
        )
    }

    #[cfg(target_os = "linux")]
    pub fn open_command(process_name: &str) -> Self {
        Command::new(process_name.to_string(), vec![])
    }

    #[cfg(target_os = "windows")]
    pub fn open_command(process_name: &str) -> Self {
        let process_name = format!("{}{}", process_name, ".exe");
        Command::new(process_name, vec![])
    }

    #[cfg(target_os = "macos")]
    pub fn open_command(process_name: &str) -> Self {
        Command::new(process_name.to_string(), vec![])
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
