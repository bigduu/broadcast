#![allow(dead_code)]
use std::{
    sync::Arc,
    time::{self, Duration, SystemTime},
};

use tokio::{net::UdpSocket, sync::Mutex, time::sleep};
use tracing::{error, info};

use super::frame::UdpFrame;
use super::node::Node;

#[derive(Debug, Clone)]
pub struct BroadcastServer {
    pub name: String,
    pub port: u16,
    pub node: Node,
    pub node_list: Arc<Mutex<Vec<Node>>>,
    pub socket: Arc<UdpSocket>,
    pub timeout: Duration,
}

impl BroadcastServer {
    pub async fn new(name: String, port: u16) -> Self {
        let node = Node::new_self_node(name.to_string(), port);
        let socket = UdpSocket::bind(&format!("0.0.0.0:{port}"))
            .await
            .unwrap_or_else(|e| {
                error!("Failed to bind socket to port {} with error {}", port, e);
                panic!("Failed to bind socket to port {port}")
            });
        socket
            .set_multicast_loop_v4(true)
            .expect("Failed to set broadcast");
        // TODO: set multicast address from config
        socket
            .join_multicast_v4("224.0.0.1".parse().unwrap(), "0.0.0.0".parse().unwrap())
            .expect("Failed to join multicast group");
        info!("Joined multicast group 224.0.0.1 successfully");
        //TODO: set timeout from config
        BroadcastServer {
            name,
            port,
            node,
            node_list: Arc::new(Mutex::new(vec![])),
            socket: Arc::new(socket),
            timeout: Duration::from_secs(5),
        }
    }
}

impl BroadcastServer {
    pub async fn scan_node(&self) {
        let cloned = self.clone();
        tokio::spawn(async move {
            cloned.notify_node().await;
        });
        let cloned = self.clone();
        tokio::spawn(async move {
            cloned.clean_node().await;
        });
        let cloned = self.clone();
        let _ = tokio::spawn(async move {
            cloned.listen_notify().await;
        })
        .await;
    }

    async fn clean_node(&self) {
        loop {
            //TODO: set clean interval from config
            sleep(Duration::from_secs(6)).await;
            let mut node_list = self.node_list.lock().await;
            let now = SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .expect("Failed to get current time");
            node_list.retain(|node| {
                let hit_timestamp = Duration::from_millis(node.hit_timestamp as u64);
                now - hit_timestamp < self.timeout
            });
            info!(
                "In server node list: {:#?}",
                node_list
                    .iter()
                    .map(|n| format!(
                        "Node: {}, timestamp: {}",
                        n.clone().name,
                        n.clone().hit_timestamp
                    ))
                    .collect::<Vec<String>>()
            );
        }
    }

    async fn listen_notify(&self) {
        loop {
            let mut buf = vec![0u8; 1400];
            let (len, _addr) = self
                .clone()
                .socket
                .recv_from(&mut buf)
                .await
                .expect("Failed to receive broadcast");
            buf.truncate(len);
            let frame = UdpFrame::from_vec(buf);
            let node = Node::try_from(frame.data).expect("Failed to parse node");
            self.add_node(node).await;
        }
    }

    async fn notify_node(&self) {
        while let Ok(node_bytes) = self.node.clone().try_into() {
            let frame = UdpFrame::new(node_bytes);
            let data = frame.to_bytes();
            if data.len() > 1400 {
                panic!("Data is too large to send size is {}", data.len());
            }
            self.clone()
                .socket
                .send_to(data.as_slice(), &format!("224.0.0.1:{}", self.port.clone()))
                .await
                .expect("Failed to send broadcast");
            //TODO: set notify interval from config
            sleep(Duration::from_secs(3)).await;
        }
    }
}

impl BroadcastServer {
    #![allow(unused_assignments)]
    pub async fn add_node(&self, mut node: Node) {
        let mut node_list = self.node_list.lock().await;
        let mut found = false;
        for item in &mut *node_list {
            if item.name == node.name {
                found = true;
                item.update_hit_timestamp();
                return;
            }
        }
        if !found {
            node.update_hit_timestamp();
            node_list.push(node);
        }
    }

    pub async fn remove_node(&self, node: Node) {
        let mut node_list = self.node_list.lock().await;
        node_list.retain(|n| n.name != node.name);
    }

    pub async fn get_node_list(&self) -> Vec<Node> {
        let node_list = self.node_list.lock().await;
        node_list.clone()
    }

    pub async fn get_node(&self, name: String) -> Option<Node> {
        let node_list = self.node_list.lock().await;
        node_list.iter().find(|n| n.name == name).cloned()
    }

    pub async fn get_node_by_port(&self, port: u16) -> Option<Node> {
        let node_list = self.node_list.lock().await;
        node_list.iter().find(|n| n.port == port).cloned()
    }

    pub async fn get_node_by_name_and_port(&self, name: String, port: u16) -> Option<Node> {
        let node_list = self.node_list.lock().await;
        node_list
            .iter()
            .find(|n| n.name == name && n.port == port)
            .cloned()
    }
}
