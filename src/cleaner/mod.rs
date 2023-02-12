use std::time::Duration;

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use regex::Regex;
use tokio::fs::read_dir;
use tracing::{error, info, log::trace};

pub struct Cleaner {
    pub folder_name: String,
    pub file_regex: Regex,
    pub expire_time: Duration,
}

impl Cleaner {
    pub fn new(folder_name: String, file_regex: Regex, expire_time: Duration) -> Self {
        Self {
            folder_name,
            file_regex,
            expire_time,
        }
    }

    pub fn new_date(folder_name: String, expire_time: Duration) -> Self {
        Self {
            folder_name,
            file_regex: Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap(),
            expire_time,
        }
    }

    pub fn new_date_time(folder_name: String, expire_time: Duration) -> Self {
        Self {
            folder_name,
            file_regex: Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}_\d{2}_\d{2}").unwrap(),
            expire_time,
        }
    }

    pub async fn clean(&self) {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            if let Ok(mut dir) = read_dir(self.folder_name.clone()).await {
                while let Ok(Some(entry)) = dir.next_entry().await {
                    let path = entry.path();
                    let file_timestamp = path
                        .file_name()
                        .and_then(|file_name| file_name.to_str())
                        .filter(|file_name| {
                            !file_name.contains("DS") || !file_name.contains("latest")
                        })
                        .and_then(|file_name| self.file_regex.captures(file_name))
                        .and_then(|captures| captures.get(0))
                        .and_then(|capture| parse_string_to_timestamp(capture.as_str()))
                        .unwrap_or(0);
                    if file_timestamp == 0 {
                        continue;
                    }
                    let expire_time = warp_to_i64(self.expire_time.as_secs());
                    let now = Utc::now().timestamp();
                    if file_timestamp + expire_time < now {
                        trace!(
                            "Will clean file {:?} , expire time: {}/{}",
                            path,
                            expire_time + file_timestamp,
                            now
                        );
                        if let Err(e) = tokio::fs::remove_file(path).await {
                            error!("Failed to remove file with error {:?}", e);
                        }
                    }
                }
            }
        }
    }
}

fn warp_to_i64(as_secs: u64) -> i64 {
    as_secs as i64
}

fn parse_string_to_timestamp(date_time: &str) -> Option<i64> {
    let date_time = if !date_time.contains('_') {
        date_time
            .parse::<NaiveDate>()
            .ok()
            .map(|date| date.and_hms_opt(0, 0, 0))
            .unwrap_or_else(|| None)
    } else {
        NaiveDateTime::parse_from_str(date_time, "%Y-%m-%d %H_%M_%S").ok()
    };
    date_time.map(|date_time| DateTime::<Utc>::from_utc(date_time, Utc).timestamp() - 8 * 3600)
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, FixedOffset, Local, Utc};

    use super::*;

    #[test]
    fn test_warp_to_i64() {
        assert_eq!(warp_to_i64(0), 0);
        assert_eq!(warp_to_i64(1), 1);
        assert_eq!(warp_to_i64(5), 5);
    }

    #[test]
    fn test_capture_date() {
        let cleaner = Cleaner::new_date("test".to_string(), Duration::from_secs(0));
        assert_eq!(
            cleaner
                .file_regex
                .captures("192.168.31.73.2023-02-10")
                .unwrap()
                .get(0)
                .unwrap()
                .as_str(),
            "2023-02-10"
        );
    }

    #[test]
    fn test_parse_date() {
        let date_time = "2023-02-12 03_10_00";
        let date_time = parse_string_to_timestamp(date_time).unwrap();
        assert_eq!(date_time, 1676142600);
        let date_time = "2023-02-12";
        let date_time = parse_string_to_timestamp(date_time).unwrap();
        assert_eq!(date_time, 1676131200)
    }
}
