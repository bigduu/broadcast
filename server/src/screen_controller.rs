use std::{panic::catch_unwind, sync::Arc};

use actix_web::{get, web::Data};
use lazy_static::lazy_static;
use tokio::sync::{mpsc::Receiver, Mutex};
use tracing::error;

lazy_static! {
    static ref LATEST: std::sync::RwLock<String> = std::sync::RwLock::new("".to_string());
}

#[get("/screen")]
pub async fn screenshot(rx: Data<Arc<Mutex<Receiver<Vec<u8>>>>>) -> String {
    std::panic::set_hook(Box::new(|e| {
        error!("screenshot error: {:?}", e);
    }));
    if let Ok(vec) = rx.lock().await.try_recv() {
        let reuslt = catch_unwind(|| image_base64::to_base64_vec(vec));
        match reuslt {
            Ok(base64) => {
                let mut latest = LATEST.write().unwrap();
                *latest = base64.clone();
                base64
            }
            Err(_) => {
                let latest = LATEST.read().unwrap();
                latest.clone()
            }
        }
    } else {
        let latest = LATEST.read().unwrap();
        latest.clone()
    }
}
