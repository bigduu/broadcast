use std::path::Path;

use actix_web::{body, get, HttpResponse, Responder};

#[get("/screen")]
pub async fn screenshot() -> String {
    let path = Path::new(".").join("screenCapture").join("latest.png");
    image_base64::to_base64(path.to_str().unwrap())
}
