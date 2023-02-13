use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::{Datelike, Local, Timelike};
use image_compressor::{compressor::Compressor, Factor};
use screenshots::Screen;
use tokio::{
    fs::{self, write},
    sync::Mutex,
};
use tracing::{error, info};

pub struct ScreenCapture {
    pub writing_lock: Arc<Mutex<bool>>,
    folder_name: String,
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self {
            writing_lock: Arc::new(Mutex::new(false)),
            folder_name: "screenCapture".to_string(),
        }
    }

    pub async fn capture(&self) {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if let Ok(screens) = Screen::all() {
                for screen in screens {
                    if screen.display_info.is_primary {
                        let _ = self.writing_lock.lock().await;
                        self.do_capture(screen).await;
                    }
                }
            } else {
                error!("Failed to get screens");
            }
        }
    }

    async fn do_capture(&self, screen: Screen) {
        if let Ok(image) = screen.capture() {
            if let Err(e) = fs::create_dir_all(self.folder_name.clone()).await {
                error!("Failed to create dir with error {:?}", e);
            }
            self.archive_file().await;
            if let Err(e) = write(
                format!("{}/{}.png", self.folder_name, "latest"),
                image.buffer(),
            )
            .await
            {
                error!("Failed to write file with error {:?}", e);
            }

            let mut compressor = Compressor::new(
                PathBuf::from(self.folder_name.clone()).join("latest.png"),
                PathBuf::from(self.folder_name.clone()).join("latest.jpg"),
            );
            compressor.set_delete_origin(true);
            compressor.set_factor(Factor::new(10., 0.2));
            let _ = compressor.compress_to_jpg();
        } else {
            error!("Failed to capture screen");
        }
    }

    async fn archive_file(&self) {
        if let Ok(dir) = std::fs::read_dir(self.folder_name.clone()) {
            let paths: Vec<PathBuf> = dir
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .collect();

            for path in paths {
                let file_name = path
                    .file_name()
                    .and_then(|file_name| file_name.to_str())
                    .filter(|file_name| file_name.contains("latest.jpg"))
                    .unwrap_or("");
                if file_name.is_empty() {
                    continue;
                }
                let now = Local::now();
                let now_string = format!(
                    "{}-{:02}-{:02} {:02}_{:02}_{:02}",
                    now.year(),
                    now.month(),
                    now.day(),
                    now.hour(),
                    now.minute(),
                    now.second()
                );

                if let Err(e) = fs::rename(
                    format!("{}/{}", self.folder_name, file_name),
                    format!("{}/{}.jpg", self.folder_name, now_string),
                )
                .await
                {
                    error!("Failed to rename file with error {:?}", e);
                }
            }
        }
    }
}
