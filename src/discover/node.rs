use std::{
    time::{self, SystemTime},
    u128,
};

use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use serde_json::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub name: String,
    pub ipaddress: String,
    pub port: u16,
    pub hit_timestamp: u128,
}

impl Node {
    pub fn new(name: String, port: u16, hit_timestamp: u128) -> Self {
        Node {
            name,
            ipaddress: local_ip().unwrap().to_string(),
            port,
            hit_timestamp,
        }
    }

    pub fn new_self_node(name: String, port: u16) -> Self {
        Node {
            name,
            ipaddress: local_ip().unwrap().to_string(),
            port,
            hit_timestamp: 0,
        }
    }

    pub fn update_hit_timestamp(&mut self) {
        self.hit_timestamp = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("Failed to get duration since unix epoch")
            .as_millis();
    }
}

impl TryFrom<Node> for Vec<u8> {
    type Error = Error;
    fn try_from(value: Node) -> Result<Self, Self::Error> {
        serde_json::to_vec(&value)
    }
}

impl TryFrom<Vec<u8>> for Node {
    type Error = Error;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_node() {
        let node = Node::new("server".to_string(), 8080, 0);
        let node_bytes: Vec<u8> = node.try_into().unwrap();
        let node = Node::try_from(node_bytes).unwrap();
        assert_eq!(node.name, "server");
        assert_eq!(node.port, 8080);
    }
}
