use std::env::temp_dir;
use std::path::PathBuf;

use async_compression::tokio::write::GzipEncoder;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
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

pub static VALID_HOP_FILENAMES: &[&str] = &[
    "hop.yml",
    "hop.yaml",
    "hop.json",
    ".hoprc",
    ".hoprc.yml",
    ".hoprc.yaml",
    ".hoprc.json",
];

// default ignore list for tar files
static DEFAULT_IGNORE_LIST: &[&str] = &[
    "/.github",
    ".gitignore",
    ".gitmodules",
    ".DS_Store",
    "/.idea",
    "/.vscode",
];

static VALID_IGNORE_FILENAMES: &[&str] = &[".hopignore", ".gitignore"];

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HopFile {
    pub version: u64,
    pub config: HopFileConfigV1,
    #[serde(skip)]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
enum HopFileVersion {
    #[serde(rename = "1")]
    V1(HopFileConfigV1),
    #[serde(rename = "2")]
    V2(HopFileConfigV2),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HopFileConfigV1 {
    pub project: String,
    pub deployment: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HopFileConfigV2 {
    pub project: String,
    pub deployment: String,
}

impl HopFile {
    pub fn new(path: PathBuf, project: String, deployment: String) -> HopFile {
        HopFile {
            version: 1,
            config: HopFileConfigV1 {
                project,
                deployment,
            },
            path: Some(path),
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

                hop_file_content.path = Some(path);

                return Some(hop_file_content);
            }
        }

        None
    }

    pub async fn save(self) -> Option<Self> {
        let path = self.path.clone().expect("HopFile::save: path is None");

        let content =
            Self::deserialize(path.clone(), self.clone()).expect("Failed to deserialize hop file");

        let mut file = File::create(&path).await.ok()?;

        file.write_all(content.as_bytes()).await.ok()?;

        Some(self)
    }
}

// compress stuff

pub async fn compress(id: String, base_dir: PathBuf) -> Result<String, std::io::Error> {
    let archive_path = temp_dir().join(format!("hop_{}.tar.gz", id));

    // tarball gunzip stuff
    let writer = File::create(archive_path.clone()).await?;
    let writer = GzipEncoder::new(writer);
    let mut archive = TarBuilder::new(writer);
    archive.follow_symlinks(true);

    // .gitignore / .hopignore
    let found_ignore = &find_ignore_files(base_dir.clone()).await;

    let files = match found_ignore {
        Some(ignore_path) => gitignore::File::new(ignore_path)
            .unwrap()
            .included_files()
            .unwrap(),
        None => {
            println!("No ignore file found, creating a .hopignore file");

            // create a new .hopignore file and add some default ignore patterns
            let mut file = File::create(base_dir.join(".hopignore")).await?;
            file.write_all(DEFAULT_IGNORE_LIST.join("\n").as_bytes())
                .await?;
            file.shutdown().await?;

            gitignore::File::new(&base_dir.join(".hopignore").to_path_buf())
                .unwrap()
                .included_files()
                .unwrap()
        }
    };

    // add all found files to the tarball
    for entry in files {
        let relative = entry.as_path().strip_prefix(&base_dir).unwrap().to_owned();

        // debug
        println!("{:?}", relative);

        archive.append_path(relative).await?;
    }

    let mut buff = archive.into_inner().await?;
    buff.shutdown().await?;
    let mut buff = buff.into_inner();
    buff.shutdown().await?;

    Ok(archive_path.to_str().unwrap().into())
}

async fn find_ignore_files(path: PathBuf) -> Option<PathBuf> {
    for filename in VALID_IGNORE_FILENAMES {
        let path = path.clone().join(filename);

        if fs::metadata(&path).await.is_ok() {
            return Some(path);
        }
    }

    None
}
