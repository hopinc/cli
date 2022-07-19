use std::path::PathBuf;

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
    pub version: u64,
    pub config: HopFileConfigV1,
    #[serde(skip)]
    pub path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
enum HopFileVersion {
    #[serde(rename = "1")]
    V1(HopFileConfigV1),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HopFileConfigV1 {
    pub project_id: String,
    pub deployment_id: String,
}

impl HopFile {
    pub fn new(path: PathBuf, project: String, deployment: String) -> HopFile {
        HopFile {
            version: 1,
            config: HopFileConfigV1 {
                project_id: project,
                deployment_id: deployment,
            },
            path,
        }
    }

    fn deserialize(path: PathBuf, content: Self) -> Option<String> {
        match path.clone().extension() {
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

    fn serialize(path: PathBuf, content: &str) -> Option<Self> {
        match path.clone().extension() {
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

    pub async fn find(path: PathBuf) -> Option<Self> {
        for filename in VALID_HOP_FILENAMES {
            let path = path.clone().join(filename);

            if fs::metadata(&path).await.is_ok() {
                let content = fs::read_to_string(&path).await.ok()?;

                let mut hop_file_content: Self = Self::serialize(path.clone(), content.as_str())
                    .expect("Failed to serialize hop file");

                hop_file_content.path = path;

                return Some(hop_file_content);
            }
        }

        None
    }

    pub async fn save(self) -> Option<Self> {
        let path = self.path.clone();

        let content =
            Self::deserialize(path.clone(), self.clone()).expect("Failed to deserialize hop file");

        let mut file = File::create(&path).await.ok()?;

        file.write_all(content.as_bytes()).await.ok()?;

        log::info!("Saved hop file to {}", path.to_str().unwrap());

        Some(self)
    }
}
