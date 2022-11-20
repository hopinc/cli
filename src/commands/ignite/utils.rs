use std::collections::hash_map::HashMap;
use std::io::Write;

use anyhow::{anyhow, bail, ensure, Context, Result};
use console::Term;
use regex::Regex;
use serde_json::Value;
use tabwriter::TabWriter;

use super::types::{
    CreateDeployment, Deployment, MultipleDeployments, Premade, Premades, ScaleRequest,
    SingleDeployment, Tier, Tiers,
};
use crate::commands::containers::types::{ContainerOptions, ContainerType};
use crate::commands::ignite::create::Options;
use crate::commands::ignite::types::{
    RamSizes, Resources, RestartPolicy, ScalingStrategy, VolumeFs,
};
use crate::state::http::HttpClient;
use crate::utils::ask_question_iter;
use crate::utils::size::parse_size;

pub async fn get_all_deployments(http: &HttpClient, project_id: &str) -> Result<Vec<Deployment>> {
    let response = http
        .request::<MultipleDeployments>(
            "GET",
            &format!("/ignite/deployments?project={project_id}"),
            None,
        )
        .await?
        .ok_or_else(|| anyhow!("Failed to parse response"))?;

    Ok(response.deployments)
}

pub async fn get_deployment(http: &HttpClient, deployment_id: &str) -> Result<Deployment> {
    let response = http
        .request::<SingleDeployment>(
            "GET",
            &format!(
                "/ignite/deployments/{deployment_id}",
                deployment_id = deployment_id
            ),
            None,
        )
        .await?
        .ok_or_else(|| anyhow!("Failed to parse response"))?;

    Ok(response.deployment)
}

pub async fn create_deployment(
    http: &HttpClient,
    project_id: &str,
    config: &CreateDeployment,
) -> Result<Deployment> {
    let response = http
        .request::<SingleDeployment>(
            "POST",
            &format!("/ignite/deployments?project={project_id}"),
            Some((
                serde_json::to_vec(&config).unwrap().into(),
                "application/json",
            )),
        )
        .await?
        .ok_or_else(|| anyhow!("Failed to parse response"))?;

    Ok(response.deployment)
}

pub async fn delete_deployment(http: &HttpClient, deployment_id: &str) -> Result<()> {
    http.request::<Value>(
        "DELETE",
        &format!("/ignite/deployments/{deployment_id}",),
        None,
    )
    .await?;

    Ok(())
}

pub async fn update_deployment(
    http: &HttpClient,
    deployment_id: &str,
    config: &CreateDeployment,
) -> Result<Deployment> {
    let response = http
        .request::<SingleDeployment>(
            "PATCH",
            &format!("/ignite/deployments/{deployment_id}"),
            Some((
                serde_json::to_vec(&config).unwrap().into(),
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
        &format!("/ignite/deployments/{deployment_id}/rollouts"),
        None,
    )
    .await?
    .ok_or_else(|| anyhow!("Failed to parse response"))?;

    Ok(())
}

pub async fn promote(http: &HttpClient, deployment_id: &str, build_id: &str) -> Result<()> {
    http.request::<Value>(
        "POST",
        &format!("/ignite/deployments/{deployment_id}/promote/{build_id}"),
        None,
    )
    .await?
    .ok_or_else(|| anyhow!("Failed to parse response"))?;

    Ok(())
}

pub async fn scale(http: &HttpClient, deployment_id: &str, scale: u64) -> Result<()> {
    http.request::<Value>(
        "PATCH",
        &format!("/ignite/deployments/{deployment_id}/scale"),
        Some((
            serde_json::to_vec(&ScaleRequest { scale }).unwrap().into(),
            "application/json",
        )),
    )
    .await?;
    // .ok_or_else(|| anyhow!("Failed to parse response"))?;

    Ok(())
}

pub async fn get_tiers(http: &HttpClient) -> Result<Vec<Tier>> {
    let response = http
        .request::<Tiers>("GET", "/ignite/tiers", None)
        .await?
        .ok_or_else(|| anyhow!("Failed to parse response"))?;

    Ok(response.tiers)
}

pub async fn get_premade(http: &HttpClient) -> Result<Vec<Premade>> {
    let response = http
        .request::<Premades>("GET", "/ignite/premade", None)
        .await?
        .ok_or_else(|| anyhow!("Failed to parse response"))?;

    Ok(response.premades)
}

pub fn format_deployments(deployments: &Vec<Deployment>, title: bool) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(
            &mut tw,
            "NAME\tID\tCONTAINERS\tCREATED\tTYPE\tSTRATEGY\tRESTART"
        )
        .unwrap();
    }

    for deployment in deployments {
        writeln!(
            &mut tw,
            "{}\t{}\t{}/{}\t{}\t{}\t{}\t{}",
            deployment.name,
            deployment.id,
            deployment.container_count,
            deployment.target_container_count,
            deployment.created_at,
            deployment.config.type_,
            deployment.config.container_strategy,
            deployment
                .config
                .restart_policy
                .as_ref()
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string()),
        )
        .unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}

pub async fn update_deployment_config(
    http: &HttpClient,
    options: Options,
    is_visual: bool,
    deployment: &Deployment,
    fallback_name: &Option<String>,
    is_update: bool,
) -> Result<(CreateDeployment, ContainerOptions)> {
    let mut config = CreateDeployment::from_deployment(deployment);
    let mut container_options = ContainerOptions::from_deployment(deployment);

    if is_visual {
        update_config_visual(
            http,
            options,
            &mut config,
            &mut container_options,
            fallback_name,
            is_update,
        )
        .await
    } else {
        update_config_args(
            http,
            options,
            &mut config,
            &mut container_options,
            is_update,
        )
        .await
    }
}

async fn update_config_args(
    http: &HttpClient,
    options: Options,
    deployment_config: &mut CreateDeployment,

    container_options: &mut ContainerOptions,
    is_update: bool,
) -> Result<(CreateDeployment, ContainerOptions)> {
    let name = options
        .config
        .name
        .or_else(|| {
            if is_update {
                deployment_config.name.clone()
            } else {
                None
            }
        })
        .expect("The argument '--name <NAME>' requires a value but none was supplied")
        .to_lowercase();

    ensure!(
        validate_deployment_name(&name),
        "Invalid deployment name, must be alphanumeric and hyphens only"
    );

    if !is_update || deployment_config.name != Some(name.clone()) {
        deployment_config.name = Some(name);
    }

    deployment_config.image.name = options
        .image
        .or_else(|| {
            if is_update {
                Some(deployment_config.image.name.clone())
            } else {
                None
            }
        })
        .expect("Please specify an image via the `--image` flag");

    let tiers = get_tiers(http).await?;

    deployment_config.resources = {
        let mut resources = Resources::default();

        if let Some(tier) = options.config.tier {
            let tier = tiers.iter().find(|t| t.name == tier).with_context(|| {
                anyhow!("Invalid tier, please use `ignite tiers` to see available tiers")
            })?;

            resources = tier.resources.clone().into();
        }

        if let Some(cpu) = options.config.cpu {
            if let Err(why) = validate_cpu_count(&cpu) {
                bail!("{why}")
            };

            resources.vcpu = cpu;
        }

        if let Some(memory) = options.config.ram {
            if let Err(why) = parse_size(&memory) {
                bail!("{why}")
            };

            resources.ram = memory;
        }

        let def_res = Resources::default();

        if resources.vcpu == def_res.vcpu && resources.ram == def_res.ram {
            bail!("No resources specified, please specify at least one of `--tier` or `--cpu`/`--ram`")
        }

        resources
    };

    if !is_update && options.config.volume != Default::default() {
        deployment_config.volume = {
            deployment_config.type_ = Some(ContainerType::Stateful);

            let mut volume = deployment_config.volume.take().unwrap_or_default();

            if let Some(mount_path) = options.config.volume.volume_mount {
                volume.mount_path = mount_path;
            }

            if let Some(size) = options.config.volume.volume_size {
                if let Err(why) = parse_size(&size) {
                    bail!("{why}")
                };

                volume.size = size;
            }

            if let Some(file_system) = options.config.volume.volume_fs {
                volume.fs = file_system;
            }

            Some(volume)
        }
    }

    deployment_config.container_strategy = ScalingStrategy::Manual;

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
    );

    if let Some(env) = options.config.env {
        deployment_config.env.extend(
            env.iter()
                .map(|kv| (kv.0.clone(), kv.1.clone()))
                .collect::<Vec<(String, String)>>(),
        );
    }

    if options.config.rm {
        if deployment_config.volume.is_some() {
            bail!("Cannot use `--rm` with ephemeral deployments")
        }

        deployment_config.type_ = Some(ContainerType::Ephemeral);
    }

    if let Some(entry) = options.config.entrypoint {
        deployment_config.entrypoint = Some(get_entrypoint_array(&entry));
    }

    if deployment_config.type_ != Some(ContainerType::Ephemeral) {
        if let Some(policy) = options.config.restart_policy {
            deployment_config.restart_policy = Some(policy);
        } else {
            deployment_config.restart_policy = Some(RestartPolicy::OnFailure);
        }
    } else {
        deployment_config.restart_policy = None;
    }

    Ok((deployment_config.clone(), container_options.clone()))
}

async fn update_config_visual(
    http: &HttpClient,
    options: Options,
    deployment_config: &mut CreateDeployment,
    container_options: &mut ContainerOptions,
    fallback_name: &Option<String>,
    is_update: bool,
) -> Result<(CreateDeployment, ContainerOptions)> {
    let name = {
        let back_name = fallback_name
            .clone()
            .or_else(|| deployment_config.name.clone())
            .unwrap_or_default();

        dialoguer::Input::<String>::new()
            .with_prompt("Deployment name")
            .default(back_name.clone())
            .show_default(!back_name.is_empty())
            .validate_with(|name: &String| -> Result<(), &str> {
                if validate_deployment_name(name) {
                    Ok(())
                } else {
                    Err("Invalid deployment name, must be alphanumeric and hyphens only")
                }
            })
            .interact_text()?
            .trim()
            .to_string()
    };

    if !is_update || deployment_config.name != Some(name.clone()) {
        deployment_config.name = Some(name);
    } else {
        deployment_config.name = None;
    }

    deployment_config.image.name =
        // if name is "" it's using hopdeploy ie. image is created on the fly
        if options.image.is_some() && options.image.clone().unwrap() == "" {
            options.image.unwrap()
        } else {
            dialoguer::Input::<String>::new()
                .with_prompt("Image name")
                .default(deployment_config.image.name.clone())
                .show_default(!deployment_config.image.name.is_empty())
                .validate_with(|image: &String| -> Result<(), &str> {
                    if image.is_empty() {
                        Err("Please specify an image")
                    } else {
                        Ok(())
                    }
                })
                .interact_text()?
        };

    let mut tiers = get_tiers(http).await?;

    // add custom tier for users to specify their own resources
    tiers.push(Tier {
        name: "Custom".to_string(),
        description: "Customize the tier to your needs".to_string(),
        ..Default::default()
    });

    deployment_config.resources = {
        let idx = dialoguer::Select::new()
            .with_prompt("Select a tier that will suit you well")
            .default(0)
            .items(&tiers.iter().map(|t| t.to_string()).collect::<Vec<String>>())
            .interact()?;

        if idx == tiers.len() - 1 {
            Term::stderr().clear_last_lines(1)?;

            let mut resources = Resources::default();

            resources.vcpu = dialoguer::Input::<f64>::new()
                .with_prompt("CPUs")
                .default(deployment_config.resources.vcpu)
                .show_default(is_update)
                .validate_with(validate_cpu_count)
                .interact_text()?;

            resources.ram = ask_question_iter(
                "Memory",
                &RamSizes::values(),
                Some(deployment_config.resources.ram.parse().unwrap_or_default()),
            )?
            .to_string();

            resources
        } else {
            tiers[idx].resources.clone().into()
        }
    };

    // default deployments to be persistent unless this is an update
    deployment_config.type_ = Some(deployment_config.type_.take().unwrap_or_default());

    if !is_update
        && dialoguer::Confirm::new()
            .with_prompt("Would you like to attach a volume?")
            .default(false)
            .interact()?
    {
        // remove the previous line for the question
        Term::stderr().clear_last_lines(1)?;

        deployment_config.type_ = Some(ContainerType::Stateful);

        let mut volume = deployment_config.volume.clone().unwrap_or_default();

        volume.size = dialoguer::Input::<String>::new()
            .with_prompt("Volume size")
            .default(volume.size)
            .show_default(is_update)
            .validate_with(|size: &String| -> Result<()> { parse_size(size).map(|_| ()) })
            .interact_text()?;

        volume.fs = ask_question_iter("Filesystem", &VolumeFs::values(), Some(volume.fs.clone()))?;

        volume.mount_path = dialoguer::Input::<String>::new()
            .with_prompt("Mount path")
            .default(volume.mount_path)
            .interact_text()?;

        deployment_config.volume = Some(volume);
    }

    deployment_config.container_strategy = ScalingStrategy::Manual;

    container_options.containers = Some(
        dialoguer::Input::<u64>::new()
            .with_prompt("Container amount to start")
            .default(container_options.containers.unwrap_or(1))
            .validate_with(|containers: &u64| -> Result<(), &str> {
                if *containers > 10 {
                    Err("Container amount must be less than or equal to 10")
                } else {
                    Ok(())
                }
            })
            .interact_text()?,
    );

    deployment_config.env = get_multiple_envs()?;

    if dialoguer::Confirm::new()
        .with_prompt("Do you want to change advanced settings?")
        .default(false)
        .interact_opt()?
        .unwrap_or(false)
    {
        if deployment_config.type_ != Some(ContainerType::Stateful)
            && dialoguer::Confirm::new()
                .with_prompt("Would you like your containers to be deleted when they exit?")
                .default(false)
                .interact()?
        {
            deployment_config.type_ = Some(ContainerType::Ephemeral);
        }

        if dialoguer::Confirm::new()
            .with_prompt("Do you want to specify a custom entrypoint?")
            .default(false)
            .interact()?
        {
            let ep = deployment_config
                .entrypoint
                .clone()
                .unwrap_or_default()
                .join(" ");

            deployment_config.entrypoint = Some(
                dialoguer::Input::<String>::new()
                    .with_prompt("Entrypoint")
                    .show_default(is_update && !ep.is_empty())
                    .default(ep)
                    .interact_text()
                    .map(|s| get_entrypoint_array(&s))?,
            );
        }

        if deployment_config.type_ != Some(ContainerType::Ephemeral)
            && dialoguer::Confirm::new()
                .with_prompt("Do you want to specify a restart policy for your containers?")
                .default(false)
                .interact()?
        {
            deployment_config.restart_policy = Some(ask_question_iter(
                "Select a restart policy that will be used for your containers",
                &RestartPolicy::values(),
                deployment_config.restart_policy.clone(),
            )?);
        }
    } else {
        deployment_config.restart_policy = Some(RestartPolicy::OnFailure);
    }

    Ok((deployment_config.clone(), container_options.clone()))
}

fn get_multiple_envs() -> Result<HashMap<String, String>> {
    let mut env = HashMap::new();

    let confirm_ = dialoguer::Confirm::new()
        .with_prompt("Add environment variables?")
        .default(false)
        .interact_opt()?
        .unwrap_or(false);

    if !confirm_ {
        return Ok(env);
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
            .default(false)
            .interact_opt()?
            .unwrap_or(false);

        if !confirm {
            break;
        }
    }

    Ok(env)
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

fn validate_deployment_name(name: &str) -> bool {
    let regex = Regex::new(r"(?i)^[a-z0-9-]{1,}$").unwrap();

    regex.is_match(name)
}

fn validate_cpu_count(cpu: &f64) -> Result<(), &'static str> {
    if cpu < &0.5 {
        Err("CPUs must be at least 0.5")
    } else if cpu % 0.5 != 0.0 {
        Err("CPUs must be a multiple of 0.5")
    } else {
        Ok(())
    }
}

fn get_entrypoint_array(entrypoint: &str) -> Vec<String> {
    let regex = Regex::new(r#"".*"|[^\s]+"#).unwrap();

    regex
        .find_iter(entrypoint)
        .map(|m| m.as_str().to_string())
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_entrypoint_array() {
        let entrypoint = r#"/bin/bash -c "echo hello world""#;

        let mut entrypoint_array = get_entrypoint_array(entrypoint).into_iter();

        assert_eq!(entrypoint_array.next(), Some("/bin/bash".to_string()));
        assert_eq!(entrypoint_array.next(), Some("-c".to_string()));
        assert_eq!(
            entrypoint_array.next(),
            Some(r#""echo hello world""#.to_string())
        );
        assert_eq!(entrypoint_array.next(), None);
    }
}
