#![allow(dead_code)]
use std::{
    sync::Arc,
    thread,
    time::{self, Duration, SystemTime},
};

use config::model::Config;
use domain::{node::Node, udp_frame::UDPFrame};
use tokio::{net::UdpSocket, sync::RwLock, time::sleep};
use tracing::{error, info, trace};
use utils::safe_get_ip;

use crate::frame_cache::FrameReceiverCache;

#[derive(Debug, Clone)]
pub struct BroadcastServer {
    pub name: String,
    pub port: u16,
    pub node: Node,
    pub node_list: Arc<RwLock<Vec<Node>>>,
    pub socket: Arc<UdpSocket>,
    pub timeout: Duration,
    frame_receiver_cache: FrameReceiverCache,
}

impl BroadcastServer {
    pub async fn from_config(config: Config) -> Self {
        let mut name = config.node_name().to_string();
        if name.is_empty() {
            name = safe_get_ip();
        }
        let mut port = config.board_port();
        if port == 0 {
            port = 8081
        }
        let mut board_ip = config.board_ip();
        if board_ip.is_empty() {
            board_ip = "224.0.0.1"
        }
        let node = Node::new_self_node(name.clone(), port);
        // TODO: try to kill port if it is already in use and try again
        let socket = UdpSocket::bind(&format!("0.0.0.0:{port}"))
            .await
            .unwrap_or_else(|e| {
                error!("Failed to bind socket to port {} with error {}", port, e);
                thread::sleep(Duration::from_secs(10));
                panic!("Failed to bind socket to port {port}")
            });
        socket
            .set_multicast_loop_v4(true)
            .expect("Failed to set broadcast");
        // TODO: set multicast address from config
        socket
            .join_multicast_v4(board_ip.parse().unwrap(), "0.0.0.0".parse().unwrap())
            .expect("Failed to join multicast group");
        info!("Joined multicast group 224.0.0.1 successfully");
        //TODO: set timeout from config
        BroadcastServer {
            name,
            port,
            node,
            node_list: Arc::new(RwLock::new(vec![])),
            socket: Arc::new(socket),
            timeout: Duration::from_secs(5),
            frame_receiver_cache: FrameReceiverCache::new(),
        }
    }

    pub async fn new(name: String, port: u16) -> Self {
        let node = Node::new_self_node(name.to_string(), port);
        // TODO: try to kill port if it is already in use and try again
        let socket = UdpSocket::bind(&format!("0.0.0.0:{port}"))
            .await
            .unwrap_or_else(|e| {
                error!("Failed to bind socket to port {} with error {}", port, e);
                thread::sleep(Duration::from_secs(10));
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
            node_list: Arc::new(RwLock::new(vec![])),
            socket: Arc::new(socket),
            timeout: Duration::from_secs(5),
            frame_receiver_cache: FrameReceiverCache::new(),
        }
    }
}

impl BroadcastServer {
    pub async fn scan_node(&self) {
        let cloned = Arc::new(self.clone());
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
            let mut node_list = self.node_list.write().await;
            let now = match SystemTime::now().duration_since(time::UNIX_EPOCH) {
                Ok(now) => now,
                Err(e) => {
                    error!("Failed to get current time with error {}", e);
                    continue;
                }
            };
            node_list.retain(|node| {
                let hit_timestamp = Duration::from_millis(node.hit_timestamp as u64);
                now - hit_timestamp < self.timeout
            });
            info!(
                "In server node list: {:#?}",
                node_list
                    .iter()
                    .map(|n| {
                        let n = n.clone();
                        format!(
                            "Node: name: {},ip: {}, timestamp: {}",
                            n.name, n.ipaddress, n.hit_timestamp
                        )
                    })
                    .collect::<Vec<String>>()
            );

            let mut config = config::get_config().await;
            config.set_node_list(node_list.clone()).await;
        }
    }

    async fn listen_notify(&self) {
        loop {
            if let Some(frame) = self.receive_frame().await {
                if let Ok(node) = Node::try_from(&frame.data) {
                    self.add_node(node).await;
                    let mut config = config::get_config().await;
                    {
                        config
                            .set_node_list(self.node_list.read().await.clone())
                            .await;
                    }
                }
            }
        }
    }

    async fn notify_node(&self) {
        while let Ok(node_bytes) = self.node.clone().try_into() {
            let frame = UDPFrame::new(node_bytes);
            self.send_frame(frame).await;
            //TODO: set notify interval from config
            sleep(Duration::from_secs(3)).await;
        }
    }

    async fn send_frame(&self, frame: UDPFrame) {
        let frames = frame.split_frame();
        for frame in frames {
            let frame_bytes = frame.to_bytes();
            let frame_bytes = frame_bytes.as_slice();
            trace!("Send frame: {:?}", frame_bytes.len());
            match self
                .clone()
                .socket
                .send_to(frame_bytes, &format!("224.0.0.1:{}", self.port.clone()))
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to send broadcast with error {}", e)
                }
            }
        }
    }

    async fn receive_frame(&self) -> Option<UDPFrame> {
        let mut buf = vec![0u8; 1500];
        let recive = self.clone().socket.recv_from(&mut buf).await;
        let (len, _addr) = match recive {
            Ok((len, addr)) => (len, addr),
            Err(e) => {
                error!("Failed to receive broadcast with error {}", e);
                return None;
            }
        };
        buf.truncate(len);
        match UDPFrame::from_vec(buf) {
            Some(frame) => self
                .frame_receiver_cache
                .is_complete(frame)
                .await
                .map(UDPFrame::merge_frames),
            None => None,
        }
    }
}

impl BroadcastServer {
    #![allow(unused_assignments)]
    pub async fn add_node(&self, mut node: Node) {
        let mut node_list = self.node_list.write().await;
        let mut found = false;
        for item in node_list.iter_mut() {
            if item.ipaddress == node.ipaddress {
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
        let mut node_list = self.node_list.write().await;
        node_list.retain(|n| n.name != node.name);
    }

    pub async fn get_node_list(&self) -> Vec<Node> {
        let node_list = self.node_list.read().await;
        node_list.clone()
    }

    pub async fn get_node(&self, name: String) -> Option<Node> {
        let node_list = self.node_list.read().await;
        node_list.iter().find(|n| n.name == name).cloned()
    }

    pub async fn get_node_by_port(&self, port: u16) -> Option<Node> {
        let node_list = self.node_list.read().await;
        node_list.iter().find(|n| n.port == port).cloned()
    }

    pub async fn get_node_by_name_and_port(&self, name: String, port: u16) -> Option<Node> {
        let node_list = self.node_list.read().await;
        node_list
            .iter()
            .find(|n| n.name == name && n.port == port)
            .cloned()
    }
}
