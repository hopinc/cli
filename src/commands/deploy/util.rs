use std::collections::HashMap;
use std::env::temp_dir;
use std::error::Error;
use std::path::PathBuf;
use std::vec;

use async_compression::tokio::write::GzipEncoder;
use regex::Regex;
use serde::Serialize;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio_tar::Builder as TarBuilder;

use crate::commands::ignite::create::DeploymentConfig;
use crate::commands::ignite::types::{
    ContainerType, CreateDeployment, RamSizes, Resources, ScalingStrategy,
};
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

pub async fn create_deployment_config(
    config: DeploymentConfig,
    fallback_name: Option<String>,
) -> CreateDeployment {
    let default = CreateDeployment::default();
    let name = config.name.clone();

    if config != DeploymentConfig::default() {
        let mut deployment = default;

        deployment.name =
            name.expect("The argument '--name <NAME>' requires a value but none was supplied");

        if !validate_deployment_name(deployment.name.clone()) {
            panic!("Invalid deployment name, must be alphanumeric and hyphens only");
        }

        deployment.container_type = config.container_type.expect(
            "The argument '--type <CONTAINER_TYPE>' requires a value but none was supplied",
        );

        deployment.container_strategy = config.scaling_strategy.expect(
            "The argument '--scaling <SCALING_STRATEGY>' requires a value but none was supplied",
        );

        deployment.resources.cpu = config
            .cpu
            .expect("The argument '--cpu <CPU>' requires a value but none was supplied");

        deployment.resources.ram = config
            .ram
            .expect("The argument '--ram <RAM>' requires a value but none was supplied")
            .to_string();

        if let Some(env) = config.env {
            deployment.env.extend(
                env.iter()
                    .map(|kv| (kv.0.clone(), kv.1.clone()))
                    .collect::<Vec<(String, String)>>(),
            );
        }

        return deployment;
    }

    info!("No config provided, running interactive mode");

    let name = dialoguer::Input::<String>::new()
        .with_prompt("Deployment name")
        .default(fallback_name.clone().unwrap_or(String::new()))
        .show_default(fallback_name.is_some())
        .validate_with(|name: &String| -> Result<(), &str> {
            if validate_deployment_name(name.to_string()) {
                Ok(())
            } else {
                Err("Invalid deployment name, must be alphanumeric and hyphens only")
            }
        })
        .interact_text()
        .unwrap();

    let container_type = ask_question_iter(
        "Container type",
        ContainerType::values(),
        ContainerType::default(),
    );

    let container_strategy = ask_question_iter(
        "Scaling strategy",
        ScalingStrategy::values(),
        ScalingStrategy::default(),
    );

    let cpu = dialoguer::Input::<u64>::new()
        .with_prompt("CPUs")
        .default(default.resources.cpu)
        .validate_with(|cpu: &u64| -> Result<(), &str> {
            if *cpu > 0 && *cpu <= 64 {
                Ok(())
            } else {
                Err("CPUs must be greater than 0 and less than or equal to 64")
            }
        })
        .interact_text()
        .unwrap();

    let ram = ask_question_iter("RAM", RamSizes::values(), RamSizes::default()).to_string();

    CreateDeployment {
        image: default.image,
        name,
        container_strategy,
        env: get_multiple_envs(),
        resources: Resources {
            cpu,
            ram,
            vgpu: vec![],
        },
        container_type,
    }
}

pub fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
    U: std::str::FromStr,
    U::Err: Error + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

fn get_multiple_envs() -> HashMap<String, String> {
    let mut env = HashMap::new();

    let confirm_ = dialoguer::Confirm::new()
        .with_prompt("Add environment variables?")
        .default(false)
        .interact_opt()
        .expect("Failed to ask for environment variables")
        .unwrap_or(false);

    if !confirm_ {
        return env;
    }

    loop {
        let ekv = get_env_from_input();

        if ekv.is_some() {
            let (key, value) = ekv.unwrap();
            env.insert(key, value);
        } else {
            break;
        }

        let continue_ = dialoguer::Confirm::new()
            .with_prompt("Add another environment variable?")
            .interact_opt()
            .expect("Failed to ask for environment variables");

        if continue_.is_none() || !continue_.unwrap() {
            break;
        }
    }

    env
}

pub async fn env_file_to_map(path: PathBuf) -> HashMap<String, String> {
    let mut env = HashMap::new();

    if !path.exists() {
        panic!("Could not find .env file at {}", path.display());
    }

    let file = fs::read_to_string(path).await.unwrap();
    let lines = file.lines();

    for line in lines {
        // ignore comments
        if line.starts_with("#") {
            continue;
        }

        match parse_key_val(line) {
            Ok((key, value)) => {
                env.insert(key, value);
            }
            Err(e) => warn!("Failed to parse env file line: {}", e),
        }
    }

    env
}

fn get_env_from_input() -> Option<(String, String)> {
    let key = dialoguer::Input::<String>::new()
        .with_prompt("Key")
        .interact_text();

    let key = match key {
        Ok(key) => key,
        Err(_) => return None,
    };

    let value = dialoguer::Input::<String>::new()
        .with_prompt("Value")
        .interact_text();

    let value = match value {
        Ok(value) => value,
        Err(_) => return None,
    };

    Some((key, value))
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
