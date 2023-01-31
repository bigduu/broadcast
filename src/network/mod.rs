use std::net::IpAddr;

use local_ip_address::list_afinet_netifas;
use serde::{Deserialize, Serialize};

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

/// List all local ipv4 ip addresses.
pub fn list_ipv4_addresses() -> Vec<NetworkInterface> {
    let network_interfaces = list_afinet_netifas().expect("Failed to list network interfaces");
    network_interfaces
        .iter()
        .filter(|(_name, ip)| ip.is_ipv4())
        .map(|(name, ip)| NetworkInterface::new(name.to_string(), *ip))
        .collect()
}
