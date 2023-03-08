use local_ip_address::{list_afinet_netifas, local_ip};
use mac_address::{name_by_mac_address, MacAddress, MacAddressIterator};
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

pub fn get_mac_address_with_name() -> Vec<(String, MacAddress)> {
    MacAddressIterator::new()
        .unwrap()
        .map(|mac| (name_by_mac_address(&mac).unwrap(), mac))
        .map(|(name, mac)| (name.unwrap(), mac))
        .collect()
}

pub fn get_mac_address() -> Vec<String> {
    MacAddressIterator::new()
        .unwrap()
        .map(|it| it.to_string())
        .collect()
}
