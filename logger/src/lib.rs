use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, Layer},
    prelude::__tracing_subscriber_SubscriberExt,
};

pub fn init_tracing(folder_name: &str, log_prefix: &str) -> WorkerGuard {
    let file_appender = tracing_appender::rolling::daily(folder_name, log_prefix);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = fmt::Subscriber::builder()
        .with_ansi(false)
        .with_max_level(tracing::Level::DEBUG)
        .finish()
        .with(Layer::default().with_writer(non_blocking).with_ansi(false));
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    info!("Initialized tracing");
    guard
}
