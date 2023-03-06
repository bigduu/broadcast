use std::{collections::HashMap, sync::Arc, time};

use domain::udp_frame::UDPFrame;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct FrameReceiverCacheKey {
    id: String,
    order_count: u8,
}

type InnderCache =
    Arc<Mutex<HashMap<FrameReceiverCacheKey, (time::Instant, Arc<Mutex<Vec<UDPFrame>>>)>>>;

#[derive(Debug, Clone)]
pub struct FrameReceiverCache {
    cache: InnderCache,
}

impl FrameReceiverCache {
    pub(crate) fn new() -> Self {
        FrameReceiverCache {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) async fn is_complete(&self, frame: UDPFrame) -> Option<Vec<UDPFrame>> {
        if frame.order_count == 0 {
            return Some(vec![frame]);
        }
        self.clean_timeout_cache().await;
        let mut cache = self.cache.lock().await;
        let key = FrameReceiverCacheKey {
            id: frame.id.clone(),
            order_count: frame.order_count,
        };
        let complete = if let Some(cache_vec) = cache.get(&key) {
            let mut cache_vec = cache_vec.1.lock().await;
            cache_vec.push(frame.clone());
            cache_vec.len() == frame.order_count as usize
        } else {
            cache.insert(
                key.clone(),
                (
                    time::Instant::now(),
                    Arc::new(Mutex::new(vec![frame.clone()])),
                ),
            );
            false
        };

        if complete {
            let cache_vec = cache.remove(&key).unwrap();
            let cache_vec = cache_vec.1.lock().await;
            Some(cache_vec.to_vec())
        } else {
            None
        }
    }

    async fn clean_timeout_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.retain(|_, v| {
            let time = v.0;
            let now = time::Instant::now();
            let duration = now.duration_since(time);
            //TODO: set timeout from config
            duration.as_secs() < 5
        });
    }
}

