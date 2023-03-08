use std::{
    sync::Arc,
    time::{self, Duration, SystemTime},
};

use domain::node::Node;
use lazy_static::lazy_static;
use tokio::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Mutex, RwLock,
    },
    time::sleep,
};
use tracing::{error, info};

lazy_static! {
    pub static ref NODE_HOLDER: NodeHoder = NodeHoder::new();
}

pub async fn run_node_holder() {
    tokio::spawn(async {
        NODE_HOLDER.clean_node().await;
    });
    tokio::spawn(async {
        NODE_HOLDER.start().await;
    });
}

pub fn get_node_holder() -> &'static NodeHoder {
    &NODE_HOLDER
}

pub async fn set_node_list(node_list: Vec<Node>) {
    NODE_HOLDER.set_node_list(node_list).await;
}

pub async fn get_node_list() -> Vec<Node> {
    NODE_HOLDER.get_node_list().await
}

pub fn get_sender() -> Sender<NodeOperation> {
    NODE_HOLDER.get_senders()
}

#[derive(Debug)]
pub enum NodeOperation {
    Remove(Node),
    InActive(Node),
    Active(Node),
    Init(Node),
}

#[derive(Debug)]
pub struct NodeHoder {
    node_list: Arc<RwLock<Vec<Node>>>,
    sender: Sender<NodeOperation>,
    receiver: Mutex<Receiver<NodeOperation>>,
    timeout: Duration,
}

impl Default for NodeHoder {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeHoder {
    pub fn new() -> Self {
        let (tx, rs) = channel(100);
        NodeHoder {
            node_list: Arc::new(RwLock::new(Vec::new())),
            sender: tx,
            receiver: Mutex::new(rs),
            timeout: Duration::from_secs(5),
        }
    }

    fn get_senders(&self) -> Sender<NodeOperation> {
        self.sender.clone()
    }

    async fn get_node_list(&self) -> Vec<Node> {
        self.node_list.read().await.clone()
    }

    async fn set_node_list(&self, node_list: Vec<Node>) {
        *self.node_list.write().await = node_list;
    }

    pub async fn add_node(&self, node: Node) -> anyhow::Result<()> {
        self.sender.send(NodeOperation::Active(node)).await?;
        Ok(())
    }

    pub async fn remove_node(&self, node: Node) -> anyhow::Result<()> {
        self.sender.send(NodeOperation::Remove(node)).await?;
        Ok(())
    }

    pub async fn update_node(&self, node: Node) -> anyhow::Result<()> {
        self.sender.send(NodeOperation::Active(node)).await?;
        Ok(())
    }

    async fn clean_node(&self) -> ! {
        loop {
            //TODO: set clean interval from config
            sleep(Duration::from_secs(6)).await;
            let node_list = self.node_list.read().await;
            let now: Duration = match SystemTime::now().duration_since(time::UNIX_EPOCH) {
                Ok(it) => it,
                Err(e) => {
                    error!("Failed to get current time with error {}", e);
                    continue;
                }
            };
            let inactivity: Vec<Node> = node_list
                .iter()
                .filter(|it| it.active)
                .filter(|node| {
                    let hit_timestamp = Duration::from_millis(node.hit_timestamp as u64);
                    now - hit_timestamp > self.timeout
                })
                .cloned()
                .collect();
            drop(node_list);
            for node in inactivity {
                self.sender
                    .send(NodeOperation::InActive(node))
                    .await
                    .unwrap_or_else(|e| {
                        error!("Failed to send node operation with error {}", e);
                    });
            }
        }
    }

    pub async fn start(&self) {
        let mut receiver = self.receiver.lock().await;
        loop {
            if let Some(operation) = receiver.recv().await {
                match operation {
                    NodeOperation::Remove(node) => {
                        let mut node_list = self.node_list.write().await;
                        node_list.retain(|it| it.id != node.id);
                        info_and_update_config(node_list.clone(), "re").await;
                    }
                    NodeOperation::InActive(mut node) => {
                        let mut node_list = self.node_list.write().await;
                        node_list.retain(|it| it.id != node.id);
                        node.inactive();
                        node_list.push(node);
                        info_and_update_config(node_list.clone(), "in").await;
                    }
                    NodeOperation::Active(mut node) => {
                        let mut node_list = self.node_list.write().await;
                        node_list.retain(|it| it.id != node.id);
                        node.active();
                        node.update_hit_timestamp();
                        node_list.push(node);
                        info_and_update_config(node_list.clone(), "ac").await;
                    }
                    NodeOperation::Init(node) => {
                        let mut node_list = self.node_list.write().await;
                        node_list.retain(|it| it.id != node.id);
                        node_list.push(node);
                        info_and_update_config(node_list.clone(), "init").await;
                    }
                }
            }
        }
    }
}

async fn info_and_update_config(vec: Vec<Node>, op: &str) {
    info!(
        "op: {}, In server node list: {:#?}",
        op,
        vec.iter()
            .filter(|n| n.active)
            .map(|n| {
                let n = n.clone();
                format!("Node: id: {}, name: {}, ip: {}", n.id, n.name, n.ipaddress)
            })
            .collect::<Vec<String>>()
    );
    tokio::spawn(async move {
        config::get_config().await.set_node_list(vec).await;
    });
}
