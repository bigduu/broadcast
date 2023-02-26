use logger::init_tracing;
use server::run;
use utils::safe_get_ip;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = init_tracing("broadcast_log", &safe_get_ip());
    let mut config = config::get_config().await;
    config.set_node_name("test".into()).await;
    run().await
}
