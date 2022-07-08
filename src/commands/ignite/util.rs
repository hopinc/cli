use std::env::temp_dir;
use std::path::PathBuf;

use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use tokio_tar::Builder as TarBuilder;

#[derive(Debug, Deserialize, Clone)]
pub struct Vgpu {
    #[serde(rename = "type")]
    pub g_type: String,
    pub count: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Resources {
    pub cpu: u64,
    pub ram: String,
    #[serde(skip)]
    pub vgpu: Vec<Vgpu>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum ContainerStrategy {
    #[serde(rename = "manual")]
    Manual,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Image {
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub enum ContainerType {
    #[serde(rename = "ephemeral")]
    Ephemeral,
    #[serde(rename = "persistent")]
    Persistent,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub version: String,
    #[serde(rename = "type")]
    pub d_type: ContainerType,
    pub image: Image,
    pub container_strategy: ContainerStrategy,
    pub resources: Resources,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub container_count: u32,
    pub config: Config,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SingleDeployment {
    pub deployment: Deployment,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MultipleDeployments {
    pub deployments: Vec<Deployment>,
}

pub static FILENAMES: &[&str] = &[
    "hop.yml",
    "hop.yaml",
    "hop.json",
    ".hoprc",
    ".hoprc.yml",
    ".hoprc.yaml",
    ".hoprc.json",
];

// default ignore list for tar files
static DEFAULT_IGNORE: &[&str] = &[
    ".git",
    ".gitignore",
    ".gitmodules",
    ".github",
    ".DS_Store",
    ".idea",
    ".vscode",
];

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HopFileContent {
    pub project_id: String,
    pub deployment: String,
}

pub async fn find_hop_file(path: PathBuf) -> Option<HopFileContent> {
    let mut dir = fs::read_dir(path.clone())
        .await
        .expect("Could not read directory");

    while let Some(entry) = dir.next_entry().await.expect("Could not read directory") {
        if let Some(filename) = entry.file_name().to_str() {
            if !FILENAMES.contains(&filename) {
                continue;
            }

            if entry
                .file_type()
                .await
                .expect("Could not get file type")
                .is_file()
            {
                let path = entry.path();

                println!("Found hop file: {}", path.display());

                let mut file = File::open(path).await.expect("Could not open file");
                let mut buffer = String::new();

                file.read_to_string(&mut buffer)
                    .await
                    .expect("Could not read file");

                let content: HopFileContent = serde_yaml::from_str(&buffer)
                    .unwrap_or_else(|_| serde_json::from_str(&buffer).unwrap());

                return Some(content);
            }
        }
    }

    None
}

// semi brokey :/
pub async fn compress(
    id: String,
    base_dir: PathBuf,
    ignore: Vec<&str>,
) -> Result<String, std::io::Error> {
    let archive_path = temp_dir().join(format!("hop_{}.tar.gz", id));
    let tar_file = File::create(&archive_path).await?;
    let ignore_list = [DEFAULT_IGNORE, FILENAMES, &ignore].concat();

    let mut archive = TarBuilder::new(tar_file);
    archive.follow_symlinks(true);

    // TODO: make custom implementation of walkdir
    let mut walker = async_walkdir::WalkDir::new(base_dir.clone());

    while let Some(entry) = walker.next().await {
        if let Ok(file) = entry {
            let relative = file.path().strip_prefix(&base_dir).unwrap().to_owned();

            if ignore_list.contains(&relative.to_str().unwrap().split("/").nth(0).unwrap()) {
                continue;
            }

            println!("{:?}", relative);

            archive.append_path(relative).await?;
        }
    }

    archive.finish().await?;

    Ok(archive_path.to_str().unwrap().into())
}
