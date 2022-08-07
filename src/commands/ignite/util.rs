use std::collections::hash_map::HashMap;
use std::error::Error;
use std::io::Write;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::Value;
use tabwriter::TabWriter;

use super::types::{
    CreateDeployment, Deployment, MultipleDeployments, ScaleRequest, SingleDeployment,
};
use crate::commands::containers::types::{ContainerOptions, ContainerType};
use crate::commands::deploy::util::validate_deployment_name;
use crate::commands::ignite::create::Options;
use crate::commands::ignite::types::{RamSizes, ScalingStrategy};
use crate::state::http::HttpClient;

pub async fn get_all_deployments(http: &HttpClient, project_id: &str) -> Result<Vec<Deployment>> {
    let response = http
        .request::<MultipleDeployments>(
            "GET",
            &format!("/ignite/deployments?project={}", project_id),
            None,
        )
        .await?
        .ok_or_else(|| anyhow!("Failed to parse response"))?;

    Ok(response.deployments)
}

pub async fn create_deployment(
    http: &HttpClient,
    project_id: &str,
    config: &CreateDeployment,
) -> Result<Deployment> {
    let response = http
        .request::<SingleDeployment>(
            "POST",
            &format!("/ignite/deployments?project={}", project_id),
            Some((
                serde_json::to_string(&config).unwrap().into(),
                "application/json",
            )),
        )
        .await?
        .ok_or_else(|| anyhow!("Failed to parse response"))?;

    Ok(response.deployment)
}

pub async fn update_deployment(
    http: &HttpClient,
    project_id: &str,
    deployment_id: &str,
    config: &CreateDeployment,
) -> Result<Deployment> {
    let response = http
        .request::<SingleDeployment>(
            "PATCH",
            &format!(
                "/ignite/deployments/{}?project={}",
                deployment_id, project_id
            ),
            Some((
                serde_json::to_string(&config).unwrap().into(),
                "application/json",
            )),
        )
        .await?
        .ok_or_else(|| anyhow!("Failed to parse response"))?;

    Ok(response.deployment)
}

pub async fn rollout(http: &HttpClient, deployment_id: &str) -> Result<()> {
    http.request::<Value>(
        "POST",
        &format!("/ignite/deployments/{}/rollouts", deployment_id),
        None,
    )
    .await?
    .ok_or_else(|| anyhow!("Failed to parse response"))?;

    Ok(())
}

pub async fn scale(http: &HttpClient, deployment_id: &str, scale: u64) -> Result<()> {
    http.request::<()>(
        "PATCH",
        &format!("/ignite/deployments/{}/scale", deployment_id),
        Some((
            serde_json::to_string(&ScaleRequest { scale })
                .unwrap()
                .into(),
            "application/json",
        )),
    )
    .await?;
    // .ok_or_else(|| anyhow!("Failed to parse response"))?;

    Ok(())
}

pub fn format_deployments(deployments: &Vec<Deployment>, title: bool) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(&mut tw, "NAME\tID\tCONTAINERS\tCREATED").unwrap();
    }

    for deployment in deployments {
        writeln!(
            &mut tw,
            "{}\t{}\t{}\t{}",
            deployment.name, deployment.id, deployment.container_count, deployment.created_at,
        )
        .unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}

pub fn update_deployment_config(
    options: Options,
    is_not_guided: bool,
    deployment: &Deployment,
    fallback_name: &Option<String>,
) -> (CreateDeployment, ContainerOptions) {
    let mut config = CreateDeployment::from_deployment(deployment);
    let mut container_options = ContainerOptions::from_deployment(deployment);

    if is_not_guided {
        update_config_from_args(options, &mut config, &mut container_options)
    } else {
        update_config_from_guided(options, &mut config, &mut container_options, fallback_name)
    }
}

fn update_config_from_args(
    options: Options,
    deployment_config: &mut CreateDeployment,
    container_options: &mut ContainerOptions,
) -> (CreateDeployment, ContainerOptions) {
    let is_update = deployment_config.clone() != CreateDeployment::default()
        || container_options.clone() != ContainerOptions::default();

    deployment_config.name = options
        .config
        .name
        .or_else(|| {
            if is_update {
                Some(deployment_config.name.clone())
            } else {
                None
            }
        })
        .expect("The argument '--name <NAME>' requires a value but none was supplied")
        .to_lowercase();

    assert!(
        validate_deployment_name(&deployment_config.name),
        "Invalid deployment name, must be alphanumeric and hyphens only"
    );

    deployment_config.image.name = options
        .image
        .or_else(|| {
            if is_update {
                Some(deployment_config.image.name.clone())
            } else {
                None
            }
        })
        .expect("The argument '--image <IMAGE>' requires a value but none was supplied");

    deployment_config.container_type = options
        .config
        .container_type
        .or_else(|| {
            if is_update {
                Some(deployment_config.container_type.clone())
            } else {
                None
            }
        })
        .expect("The argument '--type <CONTAINER_TYPE>' requires a value but none was supplied");

    deployment_config.container_strategy = options
        .config
        .scaling_strategy
        .or_else(|| {
            if is_update {
                Some(deployment_config.container_strategy.clone())
            } else {
                None
            }
        })
        .expect(
            "The argument '--strategy <SCALING_STRATEGY>' requires a value but none was supplied",
        );

    if deployment_config.container_strategy == ScalingStrategy::Autoscaled {
        container_options.containers = None;

        container_options.min_containers = Some(
            options.config.min_containers
            .or({
                if is_update {
                    container_options.min_containers
                } else {
                    None
                }
            })
            .expect("The argument '--min-containers <MIN_CONTAINERS>' requires a value but none was supplied"),
        );

        container_options.max_containers = Some(
            options.config.max_containers
            .or({
                if is_update {
                    container_options.max_containers
                } else {
                    None
                }
            })
            .expect("The argument '--max-containers <MAX_CONTAINERS>' requires a value but none was supplied"),
        );
    } else {
        container_options.min_containers = None;
        container_options.max_containers = None;

        container_options.containers = Some(
            options
                .config
                .containers
                .or({
                    if is_update {
                        container_options.containers
                    } else {
                        None
                    }
                })
                .expect(
                "The argument '--containers <CONTAINERS>' requires a value but none was supplied",
            ),
        )
    }

    deployment_config.resources.vcpu = options
        .config
        .cpu
        .or({
            if is_update {
                Some(deployment_config.resources.vcpu)
            } else {
                None
            }
        })
        .expect("The argument '--cpu <CPU>' requires a value but none was supplied");

    assert!(
        deployment_config.resources.vcpu >= 0.1,
        "The argument '--cpu <CPU>' must be at least 0.1"
    );

    deployment_config.resources.ram = options
        .config
        .ram
        .or({
            if is_update {
                Some(RamSizes::from_str(&deployment_config.resources.ram).unwrap())
            } else {
                None
            }
        })
        .expect("The argument '--ram <RAM>' requires a value but none was supplied")
        .to_string();

    if let Some(env) = options.config.env {
        deployment_config.env.extend(
            env.iter()
                .map(|kv| (kv.0.clone(), kv.1.clone()))
                .collect::<Vec<(String, String)>>(),
        );
    }

    (deployment_config.clone(), container_options.clone())
}

fn update_config_from_guided(
    options: Options,
    deployment_config: &mut CreateDeployment,
    container_options: &mut ContainerOptions,
    fallback_name: &Option<String>,
) -> (CreateDeployment, ContainerOptions) {
    let name = fallback_name
        .clone()
        .or_else(|| Some(deployment_config.name.clone()))
        .unwrap_or_default();

    deployment_config.name = dialoguer::Input::<String>::new()
        .with_prompt("Deployment name")
        .default(name.clone())
        .show_default(!name.is_empty())
        .validate_with(|name: &String| -> Result<(), &str> {
            if validate_deployment_name(name) {
                Ok(())
            } else {
                Err("Invalid deployment name, must be alphanumeric and hyphens only")
            }
        })
        .interact_text()
        .expect("Failed to get deployment name");

    deployment_config.image.name =
        // if name is "" it's using hopdeploy ie. image is created on the fly
        if options.image.is_some() && options.image.clone().unwrap() == "" {
            options.image.unwrap()
        } else {
            dialoguer::Input::<String>::new()
                .with_prompt("Image name")
                .default(deployment_config.image.name.clone())
                .show_default(!deployment_config.image.name.is_empty())
                .interact_text()
                .expect("Failed to get image name")
        };

    deployment_config.container_type = ask_question_iter(
        "Container type",
        &ContainerType::values(),
        Some(deployment_config.container_type.clone()),
    );

    deployment_config.container_strategy = ask_question_iter(
        "Scaling strategy",
        &ScalingStrategy::values(),
        Some(deployment_config.container_strategy.clone()),
    );

    if deployment_config.container_strategy == ScalingStrategy::Autoscaled {
        container_options.containers = None;

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
        container_options.min_containers = None;
        container_options.max_containers = None;

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

    deployment_config.resources.vcpu = dialoguer::Input::<f64>::new()
        .with_prompt("CPUs")
        .default(deployment_config.resources.vcpu)
        .validate_with(|cpu: &f64| -> Result<(), &str> {
            if *cpu >= 0.1 {
                Ok(())
            } else {
                Err("CPUs must be at least 0.1")
            }
        })
        .interact_text()
        .expect("Failed to get CPUs");

    deployment_config.resources.ram = ask_question_iter(
        "RAM",
        &RamSizes::values(),
        RamSizes::from_str(&deployment_config.resources.ram).ok(),
    )
    .to_string();

    deployment_config.env = get_multiple_envs();

    (deployment_config.clone(), container_options.clone())
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
        let env_kv = get_env_from_input();

        if let Some((key, value)) = env_kv {
            env.insert(key, value);
        } else {
            break;
        }

        let confirm = dialoguer::Confirm::new()
            .with_prompt("Add another environment variable?")
            .interact_opt()
            .expect("Failed to ask for environment variables");

        if confirm.is_none() || !confirm.unwrap() {
            break;
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

fn ask_question_iter<T>(prompt: &str, choices: &[T], override_default: Option<T>) -> T
where
    T: PartialEq + Clone + Serialize + Default,
{
    let choices_txt: Vec<String> = choices
        .iter()
        .map(|c| serde_json::to_string(c).unwrap().replace('"', ""))
        .collect();

    let to_compare = match override_default {
        Some(override_default) => override_default,
        None => T::default(),
    };

    let choice = dialoguer::Select::new()
        .with_prompt(prompt)
        .default(choices.iter().position(|x| x == &to_compare).unwrap())
        .items(&choices_txt)
        .interact()
        .expect("Failed to select");

    choices[choice].clone()
}
