use std::{sync::Arc, time::Duration};

use actix_cors::Cors;
use actix_web::{
    get, middleware,
    web::{delete, get, post, Data},
    App, HttpResponse, HttpServer, Responder,
};
use discover::broadcast_server::BroadcastServer;
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

pub async fn screen_shot() -> Receiver<Vec<u8>> {
    let (tx, rx) = channel(1);
    tokio::spawn(async move {
        let capture = screen::ScreenCapture::new(tx);
        capture.capture().await;
    });
    rx
}

pub async fn clear() {
    // After use channel to send image, we don't need this
    // tokio::spawn(async {
    //     cleaner::Cleaner::new_date_time("screenCapture".to_string(), Duration::from_secs(5))
    //         .clean()
    //         .await;
    // });
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
    let receiver = screen_shot().await;
    let rx = Arc::new(Mutex::new(receiver));
    clear().await;
    let config_s = config::get_config().await;
    let node_name = config_s.node_name().to_string();
    let server = Arc::new(BroadcastServer::new(node_name, 8080).await);
    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.scan_node().await;
    });
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(Cors::permissive())
            .service(static_file())
            .service(assets_file())
            .app_data(Data::new(server.clone()))
            .app_data(Data::new(rx.clone()))
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
            .route("/open_player", get().to(open_player))
            .route("/kill_player", get().to(kill_player))
    })
    //TODO should load from config file
    .bind(("0.0.0.0", 8081))?
    .run()
    .await?;
    Ok(())
}
