use std::env::temp_dir;
use std::path::PathBuf;

use async_compression::tokio::write::GzipEncoder;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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

static VALID_IGNORE_FILES: &[&str] = &[".gitignore", ".hopignore"];

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

async fn find_ignore_files(path: PathBuf) -> Vec<String> {
    let mut ignore_list: Vec<String> = vec![];

    let mut dir = fs::read_dir(path.clone())
        .await
        .expect("Could not read directory");

    while let Some(entry) = dir.next_entry().await.expect("Could not read directory") {
        if let Some(filename) = entry.file_name().to_str() {
            if !VALID_IGNORE_FILES.contains(&filename) {
                continue;
            }

            if entry
                .file_type()
                .await
                .expect("Could not get file type")
                .is_file()
            {
                let path = entry.path();

                // debug
                println!("Found ignore file: {}", path.display());

                let mut file = File::open(path).await.expect("Could not open file");
                let mut buffer = String::new();

                file.read_to_string(&mut buffer)
                    .await
                    .expect("Could not read file");

                ignore_list.extend(buffer.lines().map(|line| line.to_string()));
            }
        }
    }

    ignore_list
}

fn walk(base: &PathBuf, ignore: &[&str], prefix: Option<PathBuf>) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    let dir = std::fs::read_dir(base.clone()).expect("Could not read directory");

    for entry in dir {
        let entry = entry.expect("Could not read directory entry");

        let path = &entry.path();

        let filename = path.file_name().unwrap().to_str().unwrap();

        // FIXME: find a better way to handle patters / matching
        if ignore.contains(&filename.clone()) {
            continue;
        }
        if ignore.contains(&format!("/{}", filename).as_str()) {
            continue;
        }

        if path.is_dir() {
            let mut subpaths = walk(
                path,
                ignore,
                Some(path.strip_prefix(base).unwrap().to_path_buf()),
            );
            paths.append(&mut subpaths);
        } else {
            let mut path = path.clone();
            if let Some(ref prefix) = prefix {
                path = prefix.join(path);
            }
            paths.push(path);
        }
    }

    paths
}

// semi brokey :/
pub async fn compress<'a>(id: String, base_dir: PathBuf) -> Result<String, std::io::Error> {
    let archive_path = temp_dir().join(format!("hop_{}.tar.gz", id));

    // tarball gunzip stuff
    let writer = File::create(archive_path.clone()).await?;
    let writer = GzipEncoder::new(writer);
    let mut archive = TarBuilder::new(writer);
    archive.follow_symlinks(true);

    // .gitignore / .hopignore
    let found_ignore = &find_ignore_files(base_dir.clone()).await;
    let found_ignore = found_ignore
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<&str>>();
    let ignore_list = [DEFAULT_IGNORE, FILENAMES, &found_ignore].concat();

    // debug
    println!("Ignoring: {:?}", ignore_list);

    // add all found files to the tarball
    for entry in walk(&base_dir.clone(), &ignore_list, None) {
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
