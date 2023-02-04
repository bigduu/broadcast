#![allow(dead_code)]
use std::{
    collections::HashMap,
    sync::Arc,
    time::{self, Duration, SystemTime},
};

use tokio::{net::UdpSocket, sync::Mutex, time::sleep};
use tracing::{error, info, trace};

use crate::model::node::Node;

use super::udp_frame::UDPFrame;

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
struct FrameReceiverCacheKey {
    id: String,
    order_count: u8,
}

type InnderCache =
    Arc<Mutex<HashMap<FrameReceiverCacheKey, (time::Instant, Arc<Mutex<Vec<UDPFrame>>>)>>>;

#[derive(Debug, Clone)]
struct FrameReceiverCache {
    cache: InnderCache,
}

impl FrameReceiverCache {
    fn new() -> Self {
        FrameReceiverCache {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn is_complete(&self, frame: UDPFrame) -> Option<Vec<UDPFrame>> {
        if frame.order_count == 0 {
            return Some(vec![frame]);
        }
        self.clean_timeout_cache().await;
        let mut cache = self.cache.lock().await;
        let key = FrameReceiverCacheKey {
            id: frame.id.clone(),
            order_count: frame.order_count,
        };
        let complete = if let Some(cache_vec) = cache.get(&key) {
            let mut cache_vec = cache_vec.1.lock().await;
            cache_vec.push(frame.clone());
            cache_vec.len() == frame.order_count as usize
        } else {
            cache.insert(
                key.clone(),
                (
                    time::Instant::now(),
                    Arc::new(Mutex::new(vec![frame.clone()])),
                ),
            );
            false
        };

        if complete {
            let cache_vec = cache.remove(&key).unwrap();
            let cache_vec = cache_vec.1.lock().await;
            Some(cache_vec.to_vec())
        } else {
            None
        }
    }

    async fn clean_timeout_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.retain(|_, v| {
            let time = v.0;
            let now = time::Instant::now();
            let duration = now.duration_since(time);
            //TODO: set timeout from config
            duration.as_secs() < 5
        });
    }
}

#[derive(Debug, Clone)]
pub struct BroadcastServer {
    pub name: String,
    pub port: u16,
    pub node: Node,
    pub node_list: Arc<Mutex<Vec<Node>>>,
    pub socket: Arc<UdpSocket>,
    pub timeout: Duration,
    frame_receiver_cache: FrameReceiverCache,
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
            frame_receiver_cache: FrameReceiverCache::new(),
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
                        format!("Node: name: {}, timestamp: {}", n.name, n.hit_timestamp)
                    })
                    .collect::<Vec<String>>()
            );
        }
    }

    async fn listen_notify(&self) {
        loop {
            if let Some(frame) = self.receive_frame().await {
                if let Ok(node) = Node::try_from(&frame.data) {
                    self.add_node(node).await;
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
        let mut node_list = self.node_list.lock().await;
        let mut found = false;
        for item in node_list.iter_mut() {
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
