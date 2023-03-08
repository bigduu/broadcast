#![allow(dead_code)]

use domain::node::Node;
use serde::{Deserialize, Serialize};

use crate::update_config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    id: i64,
    board_ip: String,
    board_port: u16,
    node_timeout: u16,
    node_name: String,
    #[serde(default)]
    node_list: Vec<Node>,
}

impl Config {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn board_ip(&self) -> &str {
        self.board_ip.as_ref()
    }

    pub fn board_port(&self) -> u16 {
        self.board_port
    }

    pub fn node_timeout(&self) -> u16 {
        self.node_timeout
    }

    pub fn node_name(&self) -> &str {
        self.node_name.as_ref()
    }

    pub fn node_list(&self) -> &Vec<Node> {
        self.node_list.as_ref()
    }

    pub async fn set_board_ip(&mut self, board_ip: String) {
        self.board_ip = board_ip;
        update_config(self.clone()).await;
    }

    pub async fn set_board_port(&mut self, board_port: u16) {
        self.board_port = board_port;
        update_config(self.clone()).await;
    }

    pub async fn set_node_timeout(&mut self, node_timeout: u16) {
        self.node_timeout = node_timeout;
        update_config(self.clone()).await;
    }

    pub async fn set_node_name(&mut self, node_name: String) {
        self.node_name = node_name;
        update_config(self.clone()).await;
    }

    pub async fn set_node_list(&mut self, node_list: Vec<Node>) {
        self.node_list = node_list;
        update_config(self.clone()).await;
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            id: utils::snowflake::SNOWFLAKE
                .lock()
                .unwrap()
                .real_time_generate(),
            board_ip: "224.0.0.1".to_string(),
            board_port: 8081,
            node_timeout: 10,
            node_name: utils::safe_get_ip(),
            node_list: Vec::new(),
        }
    }
}
