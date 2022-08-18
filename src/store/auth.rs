use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::utils::home_path;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Auth {
    pub authorized: HashMap<String, String>,
}

impl Auth {
    fn path() -> PathBuf {
        home_path(".hop/auth.json")
    }

    pub async fn new() -> Self {
        let path = Auth::path();

        match fs::metadata(path.clone()).await {
            Ok(_) => match File::open(path).await {
                Ok(mut file) => {
                    let mut buffer = String::new();
                    file.read_to_string(&mut buffer)
                        .await
                        .expect("Failed to read auth store");

                    serde_json::from_str(&buffer).unwrap()
                }

                Err(err) => {
                    panic!("Error opening auth file: {}", err)
                }
            },
            Err(_) => Self::default().save().await.unwrap(),
        }
    }

    pub async fn save(&self) -> Result<Self> {
        let path = Self::path();

        fs::create_dir_all(path.parent().unwrap())
            .await
            .expect("Failed to create auth store directory");

        let mut file = File::create(path.clone())
            .await
            .expect("Error opening auth file:");

        file.write_all(
            serde_json::to_string(&self)
                .expect("Failed to deserialize auth")
                .as_bytes(),
        )
        .await
        .expect("Failed to write auth store");

        log::debug!("Saved credentials to {}", path.display());

        Ok(self.clone())
    }
}
