#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub board_ip: String,
    pub board_port: u16,
    pub node_timeout: u16,
}

impl Config {
    pub fn board_ip(&self) -> &str {
        self.board_ip.as_ref()
    }

    pub fn board_port(&self) -> u16 {
        self.board_port
    }

    pub fn node_timeout(&self) -> u16 {
        self.node_timeout
    }
}
