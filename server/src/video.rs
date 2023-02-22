use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use command;
use futures::StreamExt;
use futures::TryStreamExt;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;
use tracing::error;
use tracing::info;

use super::client;

pub async fn video_list() -> web::Json<Vec<String>> {
    let mut video_list = Vec::new();
    Path::new(".")
        .join("video")
        .read_dir()
        .unwrap()
        .for_each(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
                //TODO should load from config file
                let file_name = format!("http://localhost:8081/video_list/{file_name}");
                video_list.push(file_name);
            }
        });
    web::Json(video_list)
}

pub async fn download_video(req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path = req.match_info().query("video");
    let path = format!("./video/{path}");
    info!("path: {:?}", path);
    Ok(NamedFile::open(path)?)
}

pub async fn upload_video(
    mut payload: Multipart,
    req: HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let filename = req.match_info().query("video");
    while let Ok(Some(mut field)) = payload.try_next().await {
        let filepath = format!("./video/{filename}");
        let clone_path = filepath.clone();
        let mut f = web_create_file(clone_path).await?;
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            f = web::block(move || f.write_all(&data).map(|_| f).unwrap()).await?;
        }
    }
    Ok(HttpResponse::Created().into())
}

async fn web_create_file(path: String) -> actix_web::Result<std::fs::File> {
    let clone_path = path.clone();
    match web::block(|| create_file(clone_path)).await {
        Ok(it) => match it {
            Ok(it) => Ok(it),
            Err(e) => {
                error!("create file error: {:?}", e);
                Err(actix_web::error::ErrorInternalServerError(e))
            }
        },
        Err(e) => {
            error!("create file error: {:?}", e);
            Err(actix_web::error::ErrorInternalServerError(e))
        }
    }
}

fn create_file(path: String) -> Result<std::fs::File, Error> {
    match std::fs::File::create(path) {
        Ok(it) => Ok(it),
        Err(e) => {
            error!("create file error: {:?}", e);
            Err(Error::new(ErrorKind::Other, "create file error"))
        }
    }
}

pub async fn delete_video(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let video = req.match_info().query("video").to_string();
    let path = format!("./video/{video}");
    if let Err(e) = tokio::fs::remove_file(path).await {
        error!("delete file error: {:?}", e);
        Err(actix_web::error::ErrorInternalServerError(e))
    } else {
        Ok(HttpResponse::Ok().into())
    }
}

pub async fn play() -> actix_web::Result<HttpResponse> {
    client::play().await;
    Ok(HttpResponse::Ok().into())
}

pub async fn pause() -> actix_web::Result<HttpResponse> {
    client::pause().await;
    Ok(HttpResponse::Ok().into())
}

pub async fn open_player() -> actix_web::Result<HttpResponse> {
    command::open_player();
    Ok(HttpResponse::Ok().into())
}

pub async fn kill_player() -> actix_web::Result<HttpResponse> {
    command::kill_player();
    Ok(HttpResponse::Ok().into())
}
