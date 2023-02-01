use discover::server::Server;
use local_ip_address::local_ip;

mod config;
mod discover;
mod network;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let _ = tokio::spawn(async move {
        Server::new(local_ip().unwrap().to_string(), 8080)
            .await
            .scan_node()
            .await;
    })
    .await;
}
