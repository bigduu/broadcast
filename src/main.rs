#![allow(dead_code, unused_imports)]
use std::{path::Path, sync::Arc, thread, time::Duration};

use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::{
    get,
    web::{delete, get, post, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_prom::PrometheusMetricsBuilder;
use discover::broadcast_server::BroadcastServer;
use local_ip_address::local_ip;
use model::command::Command;
use network::safe_get_ip;
use tokio::{
    sync::{Mutex, RwLock},
    time::sleep,
};
use tracing::{error, info};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, Layer},
    prelude::__tracing_subscriber_SubscriberExt,
};
use web::{
    file::{assets_file, download_file, static_file},
    screen::screenshot,
    video::{delete_video, download_video, pause, play, upload_video, video_list},
};

mod cleaner;
mod config;
mod discover;
mod model;
mod network;
mod screen;
mod startup;
mod web;

async fn health() -> impl Responder {
    HttpResponse::Ok().body("UP")
}

async fn index() -> impl Responder {
    HttpResponse::Found()
        .append_header(("location", "/static/index.html"))
        .finish()
}

#[get("/nodes")]
async fn get_nodes(server: Data<Arc<BroadcastServer>>) -> impl Responder {
    let nods = server.get_node_list().await;
    HttpResponse::Ok().json(nods)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = init_tracing();
    // startup::auto_launch_self();
    tokio::spawn(async {
        screen::ScreenCapture::new().capture().await;
    });
    tokio::spawn(async {
        cleaner::Cleaner::new_date_time("screenCapture".to_string(), Duration::from_secs(5))
            .clean()
            .await;
    });
    tokio::spawn(async {
        cleaner::Cleaner::new_date(
            "broadcast_log".to_string(),
            Duration::from_secs(5 * 24 * 3600),
        )
        .clean()
        .await;
    });

    let prometheus = PrometheusMetricsBuilder::new("broadcast")
        .endpoint("/metrics")
        .build()
        .unwrap();

    let server = Arc::new(BroadcastServer::new(safe_get_ip(), 8080).await);
    let clone = server.clone();
    tokio::spawn(async move {
        clone.scan_node().await;
    });
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(prometheus.clone())
            .service(static_file())
            .service(assets_file())
            .app_data(Data::new(server.clone()))
            .service(get_nodes)
            .route("/", get().to(index))
            .route("/download/{filename:.*}", get().to(download_file))
            .route("/health", get().to(health))
            .service(screenshot)
            .route("/video_list", get().to(video_list))
            .route("/video_list/{video}", get().to(download_video))
            .route("/video_list/{video}", post().to(upload_video))
            .route("/video_list/{video}", delete().to(delete_video))
            .route("/play", get().to(play))
            .route("/pause", get().to(pause))
    })
    //TODO should load from config file
    .bind(("0.0.0.0", 8081))?
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
