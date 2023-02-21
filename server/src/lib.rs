use std::{sync::Arc, time::Duration};

use ::screen::ScreenCapture;
use actix_cors::Cors;
use actix_web::{
    get,
    web::{delete, get, post, Data},
    App, HttpResponse, HttpServer, Responder,
};
use discover::broadcast_server::BroadcastServer;
use file::{assets_file, download_file, static_file};
use logger::init_tracing;
use screen_controller::screenshot;
use utils::safe_get_ip;
use video::{delete_video, download_video, pause, play, upload_video, video_list};

pub mod client;
pub mod file;
pub mod screen_controller;
pub mod video;

pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("UP")
}

pub async fn index() -> impl Responder {
    HttpResponse::Found()
        .append_header(("location", "/static/index.html"))
        .finish()
}

#[get("/nodes")]
pub async fn get_nodes(server: Data<Arc<BroadcastServer>>) -> impl Responder {
    let nods = server.get_node_list().await;
    HttpResponse::Ok().json(nods)
}

pub async fn screen_shot() {
    tokio::spawn(async {
        ScreenCapture::new().capture().await;
    });
}

pub async fn clear() {
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
}

pub async fn run() -> anyhow::Result<()> {
    let _guard = init_tracing("broadcast_log", &safe_get_ip());

    screen_shot().await;
    clear().await;

    let server = Arc::new(BroadcastServer::new(safe_get_ip(), 8080).await);
    let clone = server.clone();
    tokio::spawn(async move {
        clone.scan_node().await;
    });
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
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