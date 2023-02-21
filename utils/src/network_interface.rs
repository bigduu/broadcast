use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: IpAddr,
}

impl NetworkInterface {
    pub fn new(name: String, ip: IpAddr) -> Self {
        NetworkInterface { name, ip }
    }
}
