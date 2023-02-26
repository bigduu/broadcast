use std::sync::Arc;

use storage::Storage;
use tokio::sync::Mutex;

use crate::model::Config;

pub mod model;

lazy_static::lazy_static! {
    static ref STORAGE: Arc<Mutex<Storage<Config>>> = Arc::new(Mutex::new(Storage::new("config.json".into())));
}

pub async fn get_config() -> Config {
    let mut storage = STORAGE.lock().await;
    storage.get().await.unwrap_or_default()
}

pub(crate) async fn update_config(config: Config) {
    let mut storage = STORAGE.lock().await;
    storage.set(config).await.unwrap();
}
