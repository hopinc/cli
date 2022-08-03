use std::collections::hash_map::HashMap;
use std::error::Error;
use std::io::Write;

use serde::Serialize;
use serde_json::Value;
use tabwriter::TabWriter;

use super::types::{CreateDeployment, Deployment, MultipleDeployments, SingleDeployment};
use crate::commands::containers::types::{ContainerOptions, ContainerType};
use crate::commands::deploy::util::validate_deployment_name;
use crate::commands::ignite::create::Options;
use crate::commands::ignite::types::{RamSizes, ScalingStrategy};
use crate::state::http::HttpClient;

pub async fn get_deployments(http: HttpClient, project_id: String) -> Vec<Deployment> {
    http.request::<MultipleDeployments>(
        "GET",
        &format!("/ignite/deployments?project={}", project_id),
        None,
    )
    .await
    .expect("Error while getting deployments")
    .unwrap()
    .deployments
}

pub async fn create_deployment(
    http: HttpClient,
    project_id: String,
    config: CreateDeployment,
) -> Deployment {
    http.request::<SingleDeployment>(
        "POST",
        format!("/ignite/deployments?project={}", project_id).as_str(),
        Some((
            serde_json::to_string(&config).unwrap().into(),
            "application/json",
        )),
    )
    .await
    .expect("Error while creating deployment")
    .unwrap()
    .deployment
}

pub async fn rollout(http: HttpClient, deployment_id: String) {
    http.request::<Value>(
        "POST",
        format!("/ignite/deployments/{}/rollouts", deployment_id).as_str(),
        None,
    )
    .await
    .expect("Failed to rollout")
    .expect("Failed to rollout");
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

#[allow(clippy::too_many_lines)]
pub fn create_deployment_config(
    options: Options,
    is_not_guided: bool,
    fallback_name: &Option<String>,
) -> (CreateDeployment, ContainerOptions) {
    let mut deployment_config = CreateDeployment::default();
    let name = options.config.name.clone();

    if is_not_guided {
        deployment_config.name = name
            .expect("The argument '--name <NAME>' requires a value but none was supplied")
            .to_lowercase();

        assert!(
            validate_deployment_name(&deployment_config.name),
            "Invalid deployment name, must be alphanumeric and hyphens only"
        );

        deployment_config.image.name = options
            .image
            .expect("The argument '--image <IMAGE>' requires a value but none was supplied");

        deployment_config.container_type = options.config.container_type.expect(
            "The argument '--type <CONTAINER_TYPE>' requires a value but none was supplied",
        );

        deployment_config.container_strategy = options.config.scaling_strategy.expect(
            "The argument '--scaling <SCALING_STRATEGY>' requires a value but none was supplied",
        );

        let mut container_options = ContainerOptions {
            containers: None,
            min_containers: None,
            max_containers: None,
        };

        if deployment_config.container_strategy == ScalingStrategy::Autoscaled {
            container_options.min_containers = Some(
                options.config.min_containers
                    .expect("The argument '--min-containers <MIN_CONTAINERS>' requires a value but none was supplied"),
            );
            container_options.max_containers = Some(
                options.config.max_containers
                    .expect("The argument '--max-containers <MAX_CONTAINERS>' requires a value but none was supplied"),
            );
        } else {
            container_options.containers = Some(options.config.containers.expect(
                "The argument '--containers <CONTAINERS>' requires a value but none was supplied",
            ));
        }

        deployment_config.resources.vcpu = options
            .config
            .cpu
            .expect("The argument '--cpu <CPU>' requires a value but none was supplied");

        assert!(
            deployment_config.resources.vcpu > 0.09,
            "The argument '--cpu <CPU>' must be at least 0.1"
        );

        deployment_config.resources.ram = options
            .config
            .ram
            .expect("The argument '--ram <RAM>' requires a value but none was supplied")
            .to_string();

        if let Some(env) = options.config.env {
            deployment_config.env.extend(
                env.iter()
                    .map(|kv| (kv.0.clone(), kv.1.clone()))
                    .collect::<Vec<(String, String)>>(),
            );
        }

        return (deployment_config, container_options);
    }

    log::info!("No config provided, running interactive mode");

    deployment_config.name = dialoguer::Input::<String>::new()
        .with_prompt("Deployment name")
        .default(fallback_name.clone().unwrap_or_default())
        .show_default(fallback_name.is_some())
        .validate_with(|name: &String| -> Result<(), &str> {
            if validate_deployment_name(name) {
                Ok(())
            } else {
                Err("Invalid deployment name, must be alphanumeric and hyphens only")
            }
        })
        .interact_text()
        .expect("Failed to get deployment name");

    deployment_config.image.name = match options.image {
        Some(image) => image,
        None => dialoguer::Input::<String>::new()
            .with_prompt("Image name")
            .default(String::new())
            .show_default(false)
            .interact_text()
            .expect("Failed to get image name"),
    };

    deployment_config.container_type =
        ask_question_iter("Container type", &ContainerType::values());

    deployment_config.container_strategy =
        ask_question_iter("Scaling strategy", &ScalingStrategy::values());

    let mut container_options = ContainerOptions {
        containers: None,
        min_containers: None,
        max_containers: None,
    };

    if deployment_config.container_strategy == ScalingStrategy::Autoscaled {
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

    deployment_config.resources.vcpu = dialoguer::Input::<f64>::new()
        .with_prompt("CPUs")
        .default(deployment_config.resources.vcpu)
        .validate_with(|cpu: &f64| -> Result<(), &str> {
            if *cpu < 0.1 {
                Ok(())
            } else {
                Err("CPUs must be at least 0.1")
            }
        })
        .interact_text()
        .expect("Failed to get CPUs");

    deployment_config.resources.ram = ask_question_iter("RAM", &RamSizes::values()).to_string();

    deployment_config.env = get_multiple_envs();

    (deployment_config, container_options)
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

fn ask_question_iter<T>(prompt: &str, choices: &[T]) -> T
where
    T: PartialEq + Clone + Serialize + Default,
{
    let choices_txt: Vec<String> = choices
        .iter()
        .map(|c| serde_json::to_string(c).unwrap().replace('"', ""))
        .collect();

    let choice = dialoguer::Select::new()
        .with_prompt(prompt)
        .default(choices.iter().position(|x| x == &T::default()).unwrap())
        .items(&choices_txt)
        .interact()
        .expect("Failed to select");

    choices[choice].clone()
}
