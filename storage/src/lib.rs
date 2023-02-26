use std::path::PathBuf;

use futures::executor::block_on;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::fs::write;
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct Storage<T> {
    path: PathBuf,
    data: T,
}

impl<T> Storage<T>
    where
        T: DeserializeOwned + Serialize,
        T: Clone,
        T: Default,
{
    pub fn new(path: PathBuf) -> Self {
        block_on(Self::new_async(path))
    }

    async fn new_async(path: PathBuf) -> Self {
        match Storage::read_storage(path.clone()).await {
            Ok(data) => {
                let result = Self { path, data };
                info!("Loaded {:?} storage", result.path);
                result
            }
            Err(_) => {
                let result = Self {
                    path: path.clone(),
                    data: Default::default(),
                };
                if let Err(e) = result.flush().await {
                    error!(
                        "Failed to flush data to storage when initializing {:?} storage: {}",
                        path.clone(),
                        e
                    );
                } else {
                    info!("Initialized {:?} storage", path.clone());
                }
                result
            }
        }
    }

    pub async fn get(&mut self) -> anyhow::Result<T> {
        self.data = Storage::read_storage(self.path.clone()).await?;
        Ok(self.data.clone())
    }

    pub async fn set(&mut self, data: T) -> anyhow::Result<()> {
        Storage::update_storage(&self.path, data.clone()).await?;
        self.data = data;
        Ok(())
    }

    async fn flush(&self) -> anyhow::Result<()> {
        Storage::update_storage(&self.path, self.data.clone()).await?;
        Ok(())
    }

    async fn update_storage(path: &PathBuf, data: T) -> anyhow::Result<()> {
        let serialized = serde_json::to_string(&data)?;
        write(path, serialized).await?;
        Ok(())
    }

    //read from storage
    async fn read_storage(path: PathBuf) -> anyhow::Result<T> {
        let string = tokio::fs::read_to_string(path).await?;
        let storage: T = serde_json::from_str(&string)?;
        Ok(storage)
    }
}
