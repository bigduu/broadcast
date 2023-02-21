use futures::TryStreamExt;
use reqwest::{
    multipart::{Form, Part},
    Body, Client,
};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use tracing::{error, info};

lazy_static::lazy_static! {
    static ref CLIENT: Client = Client::new();
}

pub async fn upload_file(file_path: &str, filename: &str) -> anyhow::Result<()> {
    let file = File::open(file_path).await?;
    let multipart = Form::new().part(
        "file",
        Part::stream(Body::wrap_stream(
            FramedRead::new(file, BytesCodec::new()).map_ok(|bytes| bytes.freeze()),
        )),
    );
    CLIENT
        .post(format!("http://localhost:8081/video_list/{filename}"))
        .multipart(multipart)
        .send()
        .await?;
    Ok(())
}

pub async fn pause() {
    let result = CLIENT.get("http://localhost:8082/pause").send().await;
    match result {
        Ok(body) => {
            info!("pause: {:?}", body.text().await.unwrap());
        }
        Err(e) => {
            error!("pause: {:?}", e);
        }
    }
}

pub async fn play() {
    let result = CLIENT.get("http://localhost:8082/play").send().await;
    match result {
        Ok(body) => {
            info!("play: {:?}", body.text().await.unwrap());
        }
        Err(e) => {
            error!("play: {:?}", e);
        }
    }
}
