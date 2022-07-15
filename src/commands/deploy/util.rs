use std::collections::HashMap;
use std::env::temp_dir;
use std::path::PathBuf;
use std::vec;

use async_compression::tokio::write::GzipEncoder;
use regex::Regex;
use serde::Serialize;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio_tar::Builder as TarBuilder;

use super::DeploymentConfig;
use crate::commands::ignite::types::{
    ContainerType, CreateDeployment, Image, RamSizes, Resources, ScalingStrategy,
};
use crate::config::HOP_REGISTRY_URL;
use crate::store::hopfile::VALID_HOP_FILENAMES;
use crate::{info, warn};

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

    info!("Finding files to compress...");
    let files = match found_ignore {
        Some(ignore_path) => gitignore::File::new(ignore_path)
            .unwrap()
            .included_files()
            .unwrap(),
        None => {
            warn!("No ignore file found, creating a .hopignore file");

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
        if VALID_HOP_FILENAMES.contains(&entry.file_name().unwrap().to_str().unwrap()) {
            continue;
        }

        let path = entry.as_path().strip_prefix(&base_dir).unwrap().to_owned();

        archive.append_path_with_name(entry.as_path(), path).await?;
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

pub fn validate_deployment_name(name: String) -> bool {
    let regex = Regex::new(r"^[a-zA-Z0-9-]*$").unwrap();

    regex.is_match(&name)
}

pub fn create_deployment_config(
    config: DeploymentConfig,
    name: String,
    namespace: String,
) -> CreateDeployment {
    let default = CreateDeployment {
        container_strategy: config
            .scaling_strategy
            .clone()
            .unwrap_or(ScalingStrategy::Manual),
        container_type: config
            .container_type
            .clone()
            .unwrap_or(ContainerType::Persistent),
        name: name.clone(),
        env: config
            .env
            .clone()
            .map(|env| {
                env.iter()
                    .map(|env| {
                        let mut split = env.split("=");
                        let key = split.next().unwrap_or("");
                        let value = split.next().unwrap_or("");

                        (key.to_string(), value.to_string())
                    })
                    .collect()
            })
            .unwrap_or(HashMap::new()),
        image: Image {
            name: format!("{}/{}/{}", HOP_REGISTRY_URL, namespace.clone(), name),
        },
        resources: Resources {
            cpu: config.cpu.clone().unwrap_or(1),
            ram: config.ram.clone().unwrap_or(RamSizes::M512),
            vgpu: vec![],
        },
    };

    if config != DeploymentConfig::default() {
        if !validate_deployment_name(name) {
            panic!("Invalid deployment name, must be alphanumeric and hyphens only");
        }

        return default;
    }

    info!("No config provided, running interactive mode");

    CreateDeployment {
        name: dialoguer::Input::<String>::new()
            .with_prompt("Deployment name")
            .default(default.name)
            .validate_with(|name: &String| -> Result<(), &str> {
                if validate_deployment_name(name.to_string()) {
                    Ok(())
                } else {
                    Err("Invalid deployment name, must be alphanumeric and hyphens only")
                }
            })
            .interact()
            .unwrap(),
        container_strategy: ask_question_iter(
            "Scaling strategy",
            vec![
                ScalingStrategy::Manual,
                ScalingStrategy::Stateful,
                ScalingStrategy::Autoscaled,
            ],
            default.container_strategy,
        ),
        // TODO: ask for env kvs
        env: HashMap::new(),
        image: default.image,
        resources: Resources {
            cpu: dialoguer::Input::<u64>::new()
                .with_prompt("CPUs")
                .default(default.resources.cpu)
                .interact()
                .unwrap(),
            ram: ask_question_iter(
                "RAM",
                vec![
                    RamSizes::M128,
                    RamSizes::M256,
                    RamSizes::M512,
                    RamSizes::G1,
                    RamSizes::G2,
                    RamSizes::G4,
                    RamSizes::G8,
                    RamSizes::G16,
                    RamSizes::G32,
                    RamSizes::G64,
                ],
                default.resources.ram,
            ),
            vgpu: vec![],
        },
        container_type: ask_question_iter(
            "Container type",
            vec![ContainerType::Ephemeral, ContainerType::Persistent],
            default.container_type,
        ),
    }
}

fn ask_question_iter<T>(prompt: &str, choices: Vec<T>, default: T) -> T
where
    T: std::cmp::PartialEq + Clone + Serialize,
{
    let choices_txt: Vec<String> = choices
        .iter()
        .map(|c| serde_json::to_string(c).unwrap().replace("\"", ""))
        .collect();

    let choice = dialoguer::Select::new()
        .with_prompt(prompt)
        .default(choices.iter().position(|x| x == &default).unwrap())
        .items(&choices_txt)
        .interact()
        .unwrap();

    choices[choice].clone()
}
