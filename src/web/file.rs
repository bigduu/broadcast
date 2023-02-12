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
    Files::new("/static", ".")
        .show_files_listing()
        .path_filter(|path, _| {
            info!("path: {:?}", path.to_str().unwrap());
            let current_dir = Path::new(".").join(path);
            if current_dir.is_dir() {
                true
            } else {
                current_dir
                    .extension()
                    .filter(|ex| {
                        let ex = *ex;
                        ex != "exe" && ex != "dll" && ex != "so" && ex != "dylib" && ex != "toml"
                    })
                    .is_some()
            }
        })
}
