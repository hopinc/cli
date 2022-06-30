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
    #[serde(rename = "default_project")]
    pub project: Option<String>,

    #[serde(rename = "default_user")]
    pub user: Option<String>,
}

impl Context {
    fn path() -> PathBuf {
        get_path(CONTEXT_STORE_PATH)
    }

    pub fn default() -> Context {
        Context {
            project: None,
            user: None,
        }
    }

    pub async fn new() -> Self {
        let path = Self::path();

        if fs::metadata(path.clone()).await.is_err() {
            Self::default().save().await.unwrap()
        } else {
            match File::open(path).await {
                Ok(mut file) => {
                    let mut buffer = String::new();
                    file.read_to_string(&mut buffer)
                        .await
                        .expect("Failed to read context store");

                    let auth: Self = serde_json::from_str(&buffer).unwrap();
                    auth
                }

                Err(err) => {
                    eprintln!("Error opening context file: {}", err);
                    std::process::exit(1);
                }
            }
        }
    }

    pub async fn save(self) -> Result<Context, std::io::Error> {
        let path = Self::path();

        fs::create_dir_all(path.parent().unwrap())
            .await
            .expect("Failed to create context store directory");

        match File::create(path.clone()).await {
            Ok(mut file) => {
                file.write(
                    serde_json::to_string(&self)
                        .expect("Failed to deserialize context")
                        .as_bytes(),
                )
                .await
                .expect("Failed to write context store");

                if self.user.is_some() || self.project.is_some() {
                    println!("Saved current context to {}", path.display());
                }

                Ok(self)
            }

            Err(err) => {
                eprintln!("Error creating context file: {}", err);
                std::process::exit(1);
            }
        }
    }
}
