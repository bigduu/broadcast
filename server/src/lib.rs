use std::{sync::Arc, time::Duration};

use actix_cors::Cors;
use actix_web::{
    get, middleware,
    web::{delete, get, post, Data},
    App, HttpResponse, HttpServer, Responder,
};
use controller_config::{get_config, put_node_name};
use discover::{broadcast_server::BroadcastServer, node_holder};
use file::{assets_file, download_file, static_file};
use screen_controller::screenshot;
use tokio::sync::{
    mpsc::{channel, Receiver},
    Mutex,
};
use video::{
    delete_video, download_video, kill_player, open_player, pause, play, upload_video, video_list,
};

pub mod client;
pub mod controller_config;
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
pub async fn get_nodes() -> impl Responder {
    let nodes = node_holder::get_node_list().await;
    HttpResponse::Ok().json(nodes)
}

pub async fn screen_shot() -> Receiver<Vec<u8>> {
    let (tx, rx) = channel(1);
    tokio::spawn(async move {
        let capture = screen::ScreenCapture::new(tx);
        capture.capture().await;
    });
    rx
}

pub async fn clear() {
    cleaner::Cleaner::new_date(
        "broadcast_log".to_string(),
        Duration::from_secs(5 * 24 * 3600),
    )
    .clean()
    .await;
}

pub async fn run_broadcast_server() -> anyhow::Result<()> {
    let config = config::get_config().await;
    BroadcastServer::from_config(config).await.scan_node().await;
    Ok(())
}

async fn init() {
    let config = config::get_config().await;
    node_holder::set_node_list(config.node_list().to_vec()).await;
    tokio::spawn(run_broadcast_server());
    tokio::spawn(node_holder::run_node_holder());
    tokio::spawn(clear());
}

pub async fn run() -> anyhow::Result<()> {
    let receiver = screen_shot().await;
    let rx = Arc::new(Mutex::new(receiver));
    init().await;
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(Cors::permissive())
            .service(static_file())
            .service(assets_file())
            .app_data(Data::new(rx.clone()))
            .service(get_nodes)
            .service(get_config)
            .service(put_node_name)
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
            .route("/open_player", get().to(open_player))
            .route("/kill_player", get().to(kill_player))
    })
    //TODO should load from config file
    .bind(("0.0.0.0", 8081))?
    .run()
    .await?;
    Ok(())
}
