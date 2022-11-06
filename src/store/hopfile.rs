use std::env::current_dir;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

pub static VALID_HOP_FILENAMES: &[&str] = &[
    "hop.yml",
    "hop.yaml",
    "hop.json",
    ".hoprc",
    ".hoprc.yml",
    ".hoprc.yaml",
    ".hoprc.json",
];

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HopFile {
    pub version: u8,
    pub config: HopFileConfig,
    #[serde(skip)]
    pub path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HopFileConfig {
    pub project_id: String,
    pub deployment_id: String,
}

impl HopFile {
    pub fn new(path: PathBuf, project: String, deployment: String) -> HopFile {
        HopFile {
            version: 1,
            config: HopFileConfig {
                project_id: project,
                deployment_id: deployment,
            },
            path,
        }
    }

    fn serialize(path: PathBuf, content: Self) -> Option<String> {
        match path.extension() {
            Some(ext) => match ext.to_str() {
                Some("yml") | Some("yaml") => serde_yaml::to_string(&content).ok(),
                Some("json") => serde_json::to_string(&content).ok(),
                _ => None,
            },
            None => {
                if let Ok(s) = serde_yaml::to_string(&content) {
                    Some(s)
                } else if let Ok(s) = serde_json::to_string(&content) {
                    Some(s)
                } else {
                    None
                }
            }
        }
    }

    fn deserialize(path: PathBuf, content: &str) -> Option<Self> {
        match path.extension() {
            Some(ext) => match ext.to_str() {
                Some("yml") | Some("yaml") => serde_yaml::from_str(content).ok(),
                Some("json") => serde_json::from_str(content).ok(),
                _ => None,
            },
            None => {
                if let Ok(s) = serde_yaml::from_str(content) {
                    Some(s)
                } else if let Ok(s) = serde_json::from_str(content) {
                    Some(s)
                } else {
                    None
                }
            }
        }
    }

    // Find a hopfile in the current directory or any of its parents.
    pub async fn find(mut path: PathBuf) -> Option<Self> {
        loop {
            for filename in VALID_HOP_FILENAMES {
                let file_path = path.join(filename);

                if file_path.exists() {
                    let content = fs::read_to_string(file_path.clone()).await.ok()?;

                    return Self::deserialize(file_path, &content);
                }
            }

            if !path.pop() {
                break;
            }
        }

        None
    }

    #[inline]
    pub async fn find_current() -> Option<Self> {
        Self::find(current_dir().ok()?).await
    }

    pub async fn save(self) -> Result<Self> {
        let path = self.path.clone();

        let content =
            Self::serialize(path.clone(), self.clone()).expect("Failed to serialize hop file");

        let mut file = File::create(&path).await?;

        file.write_all(content.as_bytes()).await?;

        log::info!("Saved hop file to {}", path.display());

        Ok(self)
    }
}
