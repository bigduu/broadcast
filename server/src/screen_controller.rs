use std::path::Path;

use actix_web::get;

#[get("/screen")]
pub async fn screenshot() -> String {
    let path = Path::new(".").join("screenCapture").join("latest.jpg");
    image_base64::to_base64(path.to_str().unwrap())
}
