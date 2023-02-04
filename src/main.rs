use discover::broadcast_server::BroadcastServer;
use local_ip_address::local_ip;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt};

mod config;
mod discover;
mod model;
mod network;

#[tokio::main]
async fn main() {
    let _guard = init_tracing();
    let _ = tokio::spawn(async move {
        BroadcastServer::new(local_ip().unwrap().to_string(), 8080)
            .await
            .scan_node()
            .await;
    })
    .await;
}

fn init_tracing() -> WorkerGuard {
    let file_appender = tracing_appender::rolling::daily("log", local_ip().unwrap().to_string());
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = fmt::Subscriber::builder()
        .with_ansi(false)
        .with_max_level(tracing::Level::DEBUG)
        .finish()
        .with(
            fmt::Layer::default()
                .with_writer(non_blocking)
                .with_ansi(false),
        );
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    guard
}
