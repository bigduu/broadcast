#![allow(dead_code)]
use std::net::IpAddr;

use local_ip_address::list_afinet_netifas;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: IpAddr,
}

impl NetworkInterface {
    fn new(name: String, ip: IpAddr) -> Self {
        NetworkInterface { name, ip }
    }
}

// List all local ipv4 ip addresses.
pub fn list_ipv4_addresses() -> Vec<NetworkInterface> {
    list_afinet_netifas()
        .unwrap_or_else(|e| {
            error!("list_afinet_netifas error: {}", e);
            Vec::new()
        })
        .iter()
        .filter(|(_name, ip)| ip.is_ipv4())
        .map(|(name, ip)| NetworkInterface::new(name.clone(), *ip))
        .collect()
}
