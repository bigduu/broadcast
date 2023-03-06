#![allow(dead_code)]

use std::time;

use image_compressor::{compressor::Compressor, Factor};
use screenshots::Screen;
use tokio::{sync::mpsc::Sender, task::spawn_blocking};
use tracing::{error, trace};

#[derive(Debug)]
pub struct ScreenCapture {
    sender: Sender<Vec<u8>>,
}

impl ScreenCapture {
    pub fn new(sender: Sender<Vec<u8>>) -> Self {
        Self { sender }
    }

    pub fn get_sender(&self) -> Sender<Vec<u8>> {
        self.sender.clone()
    }

    pub async fn capture(&self) {
        loop {
            if let Ok(screens) = Screen::all() {
                for screen in screens {
                    if screen.display_info.is_primary {
                        self.do_capture_channel(screen).await;
                    }
                }
            } else {
                error!("Failed to get screens");
            }
        }
    }

    async fn do_capture_channel(&self, screen: Screen) {
        let capture_start_time = time::Instant::now();
        if let Ok(image) = spawn_blocking(move || screen.capture()).await {
            let image = image.expect("Failed to capture image");
            trace!(
                "Capture image with size: {}x{} in {}ms",
                image.width(),
                image.height(),
                capture_start_time.elapsed().as_millis()
            );
            let compress_start_time = time::Instant::now();
            let buffer = image.buffer();
            let image = self.do_compress(buffer.clone()).await;
            trace!(
                "Compress image with size: {} to {} in {}ms",
                convert_bytes_size_to_readable(buffer.len()),
                convert_bytes_size_to_readable(image.len()),
                compress_start_time.elapsed().as_millis()
            );
            if (self.sender.send(image).await).is_ok() {
                trace!("Sent image to channel");
            } else {
                error!("Failed to send image with error, may the channel is closed");
            }
        }
    }

    async fn do_compress(&self, image: Vec<u8>) -> Vec<u8> {
        spawn_blocking(|| {
            let mut compressor = Compressor::new(image);
            compressor.set_factor(Factor::new(80., 0.8));
            compressor
                .compress_image()
                .expect("Failed to compress image")
        })
        .await
        .expect("Failed to spawn blocking")
    }
}

fn convert_bytes_size_to_readable(size: usize) -> String {
    let mut size = size as f64;
    let mut unit = "B";
    if size > 1024. {
        size /= 1024.;
        unit = "KB";
    }
    if size > 1024. {
        size /= 1024.;
        unit = "MB";
    }
    if size > 1024. {
        size /= 1024.;
        unit = "GB";
    }
    format!("{:.2}{}", size, unit)
}
