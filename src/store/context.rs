// TODO: for now both auth and context have the same functions / code
// until i figure out how to use generics / inheritence in rust

use std::path::PathBuf;

use super::utils::get_path;
use crate::config::CONTEXT_STORE_PATH;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Context {
    pub default_project: Option<String>,

    #[serde(rename = "default_user")]
    pub user: Option<String>,

    #[serde(skip)]
    pub project: Option<String>,
}

impl Context {
    fn path() -> PathBuf {
        get_path(CONTEXT_STORE_PATH)
    }

    pub fn current_project(self) -> Option<String> {
        match self.project {
            Some(project) => Some(project),
            None => self.default_project,
        }
    }

    pub fn default() -> Context {
        Context {
            default_project: None,
            project: None,
            user: None,
        }
    }

    pub async fn new() -> Self {
        let path = Self::path();

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

        if !self.user.is_none() || !self.project.is_none() {
            println!("Saved context to {}", path.display());
        }

        Ok(self)
    }
}
