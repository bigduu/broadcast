#![allow(dead_code)]
use std::{sync::Arc, thread, time::Duration};

use config::model::Config;
use domain::{node::Node, udp_frame::UDPFrame};
use tokio::{net::UdpSocket, sync::Mutex, time::sleep};
use tracing::{error, info, trace};

use crate::{
    frame_cache::FrameReceiverCache,
    node_holder::{self, NodeOperation},
};

#[derive(Debug, Clone)]
pub struct BroadcastServer {
    pub port: u16,
    pub node: Arc<Mutex<Node>>,
    pub socket: Arc<UdpSocket>,
    frame_receiver_cache: FrameReceiverCache,
}

impl BroadcastServer {
    pub async fn from_config(config: Config) -> Self {
        let name = config.node_name().to_string();
        let port = config.board_port();
        let board_ip = config.board_ip();
        let id = config.id();
        let node = Node::new_self_node(id, name.clone(), port);
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
            port,
            node: Arc::new(Mutex::new(node)),
            socket: Arc::new(socket),
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
        let _ = tokio::spawn(async move {
            cloned.listen_notify().await;
        })
        .await;
    }

    async fn listen_notify(&self) {
        let sender = node_holder::get_sender();
        loop {
            if let Some(frame) = self.receive_frame().await {
                if let Ok(node) = Node::try_from(&frame.data) {
                    if let Err(e) = sender.send(NodeOperation::Active(node)).await {
                        error!("Failed to send node to node holder with error {}", e);
                    }
                }
            }
        }
    }

    async fn notify_node(&self) {
        while let Ok(node_bytes) = self.node.lock().await.clone().try_into() {
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
