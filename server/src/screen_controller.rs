use std::path::Path;

use actix_web::get;
use tracing::error;

lazy_static::lazy_static! {
    static ref LATEST: std::sync::Mutex<String> = std::sync::Mutex::new("".to_string());
}

#[get("/screen")]
pub async fn screenshot() -> String {
    let path = Path::new(".").join("screenCapture").join("latest.jpg");
    std::panic::set_hook(Box::new(|e| {
        error!("screenshot error: {:?}", e);
    }));
    let reuslt = std::panic::catch_unwind(|| image_base64::to_base64(path.to_str().unwrap()));
    match reuslt {
        Ok(base64) => {
            let mut latest = LATEST.lock().unwrap();
            *latest = base64.clone();
            base64
        }
        Err(_) => {
            let latest = LATEST.lock().unwrap();
            latest.clone()
        }
    }
}
