use std::collections::HashMap;
use std::path::PathBuf;

use super::utils::get_path;
use crate::config::AUTH_STORE_PATH;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Auth {
    pub authorized: HashMap<String, String>,
}

impl Auth {
    fn path() -> PathBuf {
        get_path(AUTH_STORE_PATH)
    }

    pub fn default() -> Auth {
        Auth {
            authorized: HashMap::new(),
        }
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

                    let auth: Self = serde_json::from_str(&buffer).unwrap();
                    auth
                }

                Err(err) => {
                    panic!("Error opening auth file: {}", err)
                }
            },
            Err(_) => Self::default().save().await.unwrap(),
        }
    }

    pub async fn save(self) -> Result<Self, std::io::Error> {
        let path = Self::path();

        fs::create_dir_all(path.parent().unwrap())
            .await
            .expect("Failed to create auth store directory");

        let mut file = File::create(path.clone())
            .await
            .expect("Error opening auth file:");

        file.write(
            serde_json::to_string(&self)
                .expect("Failed to deserialize auth")
                .as_bytes(),
        )
        .await
        .expect("Failed to write auth store");

        if !self.authorized.is_empty() {
            println!("Saved credentials to {}", path.display());
        }

        Ok(self)
    }
}
