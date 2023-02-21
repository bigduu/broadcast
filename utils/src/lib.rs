use local_ip_address::{list_afinet_netifas, local_ip};
use network_interface::NetworkInterface;
use tracing::{error, info};

pub mod network_interface;
pub mod snowflake;

pub fn safe_get_ip() -> String {
    match local_ip() {
        Ok(ip) => {
            info!("Local ip is {}", ip.to_string());
            ip.to_string()
        }
        Err(e) => {
            info!("Failed to get local ip with error {:?}", e);
            panic!("Failed to get local ip with error {:?}", e)
        }
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
