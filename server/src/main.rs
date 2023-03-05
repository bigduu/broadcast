use logger::init_tracing;
use utils::safe_get_ip;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = init_tracing("broadcast_log", &safe_get_ip());
    server::run().await
}
