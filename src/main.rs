#![allow(dead_code, unused_imports)]
use std::{path::Path, thread, time::Duration};

use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::{
    web::{delete, get, post},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use discover::broadcast_server::BroadcastServer;
use local_ip_address::local_ip;
use model::command::Command;
use startup::auto_launch_self;
use tokio::time::sleep;
use tracing::{error, info};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, Layer},
    prelude::__tracing_subscriber_SubscriberExt,
};
use web::video::{delete_video, download_video, upload_video, video_list};

mod config;
mod discover;
mod model;
mod network;
mod startup;
mod web;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("UP")
}

async fn download_file(req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path = req.match_info().query("filename");
    let path = format!("./{path}");
    info!("path: {:?}", path);
    Ok(NamedFile::open(path)?)
}

fn static_file() -> Files {
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

fn safe_get_ip() -> String {
    match std::panic::catch_unwind(|| local_ip().unwrap().to_string()) {
        Ok(ip) => ip,
        Err(e) => {
            info!("Failed to get local ip with error {:?}", e);
            thread::sleep(Duration::from_secs(5));
            local_ip().unwrap().to_string()
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = init_tracing();
    auto_launch_self();
    tokio::spawn(async move {
        BroadcastServer::new(safe_get_ip(), 8080)
            .await
            .scan_node()
            .await;
    });
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .service(static_file())
            .route("/download/{filename:.*}", get().to(download_file))
            .route("/", get().to(index))
            .route("/video_list", get().to(video_list))
            .route("/video_list/{video}", get().to(download_video))
            .route("/video_list/{video}", post().to(upload_video))
            .route("/video_list/{video}", delete().to(delete_video))
    })
    //TODO should load from config file
    .bind(("127.0.0.1", 8081))?
    .run()
    .await?;
    Ok(())
}

fn init_tracing() -> WorkerGuard {
    let file_appender = tracing_appender::rolling::daily("broadcast_log", safe_get_ip());
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = fmt::Subscriber::builder()
        .with_ansi(false)
        .with_max_level(tracing::Level::DEBUG)
        .finish()
        .with(Layer::default().with_writer(non_blocking).with_ansi(false));
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    guard
}
