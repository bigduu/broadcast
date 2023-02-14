use std::process::Command;

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
    Command::new("pause.bat")
        .output()
        .expect("failed to execute process");
}

pub async fn play() {
    Command::new("play.bat")
        .output()
        .expect("failed to execute process");
}
