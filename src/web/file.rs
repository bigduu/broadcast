use std::path::Path;

use actix_files::{Files, NamedFile};
use actix_web::{web, HttpRequest};
use tracing::info;

pub async fn download_file(req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path = req.match_info().query("filename");
    let path = format!("./{path}");
    info!("path: {:?}", path);
    Ok(NamedFile::open(path)?)
}

pub fn static_file() -> Files {
    Files::new("/static", "./static")
        .show_files_listing()
        .index_file("index.html")
}

pub fn assets_file() -> Files {
    Files::new("/assets", "./static/assets")
        .show_files_listing()
        .index_file("index.html")
}
