#![allow(dead_code)]
use std::{net::IpAddr, thread, time::Duration};

use local_ip_address::{list_afinet_netifas, local_ip};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

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

pub fn safe_get_ip() -> String {
    match std::panic::catch_unwind(|| local_ip().unwrap().to_string()) {
        Ok(ip) => ip,
        Err(e) => {
            info!("Failed to get local ip with error {:?}", e);
            thread::sleep(Duration::from_secs(5));
            local_ip().unwrap().to_string()
        }
    }
}
