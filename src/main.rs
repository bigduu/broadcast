use discover::broadcast_server::BroadcastServer;
use local_ip_address::local_ip;

mod config;
mod discover;
mod id;
mod network;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let _ = tokio::spawn(async move {
        BroadcastServer::new(local_ip().unwrap().to_string(), 8080)
            .await
            .scan_node()
            .await;
    })
    .await;
}
