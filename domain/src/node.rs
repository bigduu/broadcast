use std::{
    time::{self, SystemTime},
    u128,
};

use postcard::Error;
use serde::{Deserialize, Serialize};
use utils::{get_mac_address, safe_get_ip};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: i64,
    pub name: String,
    pub ipaddress: String,
    pub port: u16,
    pub hit_timestamp: u128,
    pub mac_address: Vec<String>,
    pub active: bool,
}

impl Node {
    pub fn new(id: i64, name: String, port: u16, hit_timestamp: u128) -> Self {
        Node {
            id,
            name,
            ipaddress: safe_get_ip(),
            port,
            hit_timestamp,
            mac_address: get_mac_address(),
            active: true,
        }
    }

    pub fn new_self_node(id: i64, name: String, port: u16) -> Self {
        Node::new(id, name, port, 0)
    }

    pub fn update_hit_timestamp(&mut self) {
        self.hit_timestamp = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("Failed to get duration since unix epoch")
            .as_millis();
    }

    pub fn update_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn active(&mut self) {
        self.active = true;
    }

    pub fn inactive(&mut self) {
        self.active = false;
    }
}

impl TryFrom<Node> for Vec<u8> {
    type Error = Error;
    fn try_from(value: Node) -> Result<Self, Self::Error> {
        postcard::to_allocvec(&value)
    }
}

impl TryFrom<&Vec<u8>> for Node {
    type Error = Error;
    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        postcard::from_bytes(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_node() {
        let node = Node::new(
            utils::snowflake::SNOWFLAKE
                .lock()
                .unwrap()
                .real_time_generate(),
            "server".to_string(),
            8080,
            0,
        );
        let node_bytes: Vec<u8> = node.try_into().unwrap();
        let node = Node::try_from(node_bytes.as_ref()).unwrap();
        assert_eq!(node.name, "server");
        assert_eq!(node.port, 8080);
    }
}
