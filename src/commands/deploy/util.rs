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

use super::types::ContainerOptions;
use crate::commands::ignite::create::DeploymentConfig;
use crate::commands::ignite::types::{
    ContainerType, CreateDeployment, RamSizes, Resources, ScalingStrategy,
};
use crate::store::hopfile::VALID_HOP_FILENAMES;

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

    log::info!("Finding files to compress...");
    let mut walker = match found_ignore {
        Some(ignore_path) => ignore::WalkBuilder::new(ignore_path.clone())
            .follow_links(false)
            .add_custom_ignore_filename(VALID_IGNORE_FILENAMES[0])
            .build(),
        None => {
            log::warn!("No ignore file found, creating a .hopignore file");

            // create a new .hopignore file and add some default ignore patterns
            let mut file = File::create(base_dir.join(".hopignore")).await?;
            file.write_all(DEFAULT_IGNORE_LIST.join("\n").as_bytes())
                .await?;
            file.shutdown().await?;

            ignore::WalkBuilder::new(&base_dir.clone())
                .follow_links(false)
                .add_custom_ignore_filename(VALID_IGNORE_FILENAMES[0])
                .build()
        }
    };

    // skip first entry
    walker.next();

    // add all found files to the tarball
    for entry in walker {
        match entry {
            Ok(entry) => {
                log::debug!("Adding {} to tarball", entry.path().display());

                if VALID_HOP_FILENAMES.contains(&entry.file_name().to_str().unwrap()) {
                    continue;
                }

                let path = entry.path().strip_prefix(&base_dir).unwrap().to_owned();

                archive.append_path_with_name(entry.path(), path).await?;
            }
            Err(err) => {
                log::warn!("Error walking: {}", err);
            }
        }
    }

    let mut buff = archive.into_inner().await?;
    buff.shutdown().await?;
    let mut buff = buff.into_inner();
    buff.shutdown().await?;

    Ok(archive_path.to_str().unwrap().into())
}

async fn find_ignore_files(path: PathBuf) -> Option<PathBuf> {
    for filename in VALID_IGNORE_FILENAMES {
        let suffixed_path = path.clone().join(filename);

        if fs::metadata(&suffixed_path).await.is_ok() {
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
    is_not_guided: bool,
    fallback_name: Option<String>,
) -> (CreateDeployment, ContainerOptions) {
    let default = CreateDeployment::default();
    let name = config.name.clone();

    if is_not_guided {
        let mut deployment_config = default;

        deployment_config.name =
            name.expect("The argument '--name <NAME>' requires a value but none was supplied");

        if !validate_deployment_name(deployment_config.name.clone()) {
            panic!("Invalid deployment name, must be alphanumeric and hyphens only");
        }

        deployment_config.container_type = config.container_type.expect(
            "The argument '--type <CONTAINER_TYPE>' requires a value but none was supplied",
        );

        deployment_config.container_strategy = config.scaling_strategy.expect(
            "The argument '--scaling <SCALING_STRATEGY>' requires a value but none was supplied",
        );

        let mut container_options = ContainerOptions {
            containers: None,
            min_containers: None,
            max_containers: None,
        };

        if deployment_config.container_strategy == ScalingStrategy::Autoscaled {
            container_options.min_containers = Some(
                config.min_containers
                    .expect("The argument '--min-containers <MIN_CONTAINERS>' requires a value but none was supplied"),
            );
            container_options.max_containers = Some(
                config.max_containers
                    .expect("The argument '--max-containers <MAX_CONTAINERS>' requires a value but none was supplied"),
            );
        } else {
            container_options.containers = Some(config.containers.expect(
                "The argument '--containers <CONTAINERS>' requires a value but none was supplied",
            ));
        }

        deployment_config.resources.cpu = config
            .cpu
            .expect("The argument '--cpu <CPU>' requires a value but none was supplied");

        if deployment_config.resources.cpu < 1 {
            panic!("The argument '--cpu <CPU>' must be at least 1");
        }

        if deployment_config.resources.cpu > 32 {
            panic!("The argument '--cpu <CPU>' must be less than or equal to 32");
        }

        deployment_config.resources.ram = config
            .ram
            .expect("The argument '--ram <RAM>' requires a value but none was supplied")
            .to_string();

        if let Some(env) = config.env {
            deployment_config.env.extend(
                env.iter()
                    .map(|kv| (kv.0.clone(), kv.1.clone()))
                    .collect::<Vec<(String, String)>>(),
            );
        }

        return (deployment_config, container_options);
    }

    log::info!("No config provided, running interactive mode");

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

    let mut container_options = ContainerOptions {
        containers: None,
        min_containers: None,
        max_containers: None,
    };

    if container_strategy == ScalingStrategy::Autoscaled {
        container_options.min_containers = Some(
            dialoguer::Input::<u64>::new()
                .with_prompt("Minimum container ammount")
                .default(1)
                .validate_with(|containers: &u64| -> Result<(), &str> {
                    if *containers > 0 {
                        Ok(())
                    } else if *containers > 10 {
                        Err("Container ammount must be less than or equal to 10")
                    } else {
                        Err("Container ammount must be greater than 0")
                    }
                })
                .interact()
                .expect("Failed to get minimum containers"),
        );
        container_options.max_containers = Some(
            dialoguer::Input::<u64>::new()
                .with_prompt("Maximum container ammount")
                .default(10)
                .validate_with(|containers: &u64| -> Result<(), &str> {
                    if *containers > 0 {
                        Ok(())
                    } else if *containers > 10 {
                        Err("Container ammount must be less than or equal to 10")
                    } else {
                        Err("Container ammount must be greater than 0")
                    }
                })
                .interact()
                .expect("Failed to get maximum containers"),
        );
    } else {
        container_options.containers = Some(
            dialoguer::Input::<u64>::new()
                .with_prompt("Container ammount")
                .default(1)
                .validate_with(|containers: &u64| -> Result<(), &str> {
                    if *containers < 1 {
                        Err("Container ammount must be at least 1")
                    } else if *containers > 10 {
                        Err("Container ammount must be less than or equal to 10")
                    } else {
                        Ok(())
                    }
                })
                .interact()
                .expect("Failed to get containers"),
        );
    }

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

    (
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
        },
        container_options,
    )
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
            Err(e) => log::warn!("Failed to parse env file line: {}", e),
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
