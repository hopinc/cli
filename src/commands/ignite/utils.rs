use std::collections::hash_map::HashMap;
use std::error::Error;
use std::io::Write;
use std::str::FromStr;

use anyhow::{anyhow, ensure, Result};
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
    RamSizes, Resources, RestartPolicy, ScalingStrategy, Volume, VolumeFs,
};
use crate::state::http::HttpClient;
use crate::utils::ask_question_iter;

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
    is_not_guided: bool,
    deployment: &Deployment,
    fallback_name: &Option<String>,
    is_update: bool,
) -> Result<(CreateDeployment, ContainerOptions)> {
    let mut config = CreateDeployment::from_deployment(deployment);
    let mut container_options = ContainerOptions::from_deployment(deployment);

    if is_not_guided {
        update_config_args(
            http,
            options,
            &mut config,
            &mut container_options,
            is_update,
        )
        .await
    } else {
        update_config_visual(
            http,
            options,
            &mut config,
            &mut container_options,
            fallback_name,
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

    assert!(
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
        .expect("The argument '--image <IMAGE>' requires a value but none was supplied");

    deployment_config.type_ = Some(
        options
            .config
            .container_type
            .or_else(|| {
                if is_update {
                    deployment_config.type_.clone()
                } else {
                    None
                }
            })
            .expect(
                "The argument '--type <CONTAINER_TYPE>' requires a value but none was supplied",
            ),
    );

    if deployment_config.type_ != Some(ContainerType::Ephemeral) {
        deployment_config.restart_policy = Some(options
                .config
                .restart_policy
                .or_else(|| {
                    if is_update {
                        deployment_config.restart_policy.clone()
                    } else {
                        None
                    }
                })
                .expect("The argument '--restart-policy <RESTART_POLICY>' requires a value but none was supplied"));
    }

    deployment_config.container_strategy = ScalingStrategy::Manual;

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

    if let Err(why) = validate_cpu_count(&deployment_config.resources.vcpu) {
        panic!("{why}")
    };

    deployment_config.resources.ram = options
        .config
        .ram
        .or({
            if is_update {
                Some(
                    RamSizes::from_str(&deployment_config.resources.ram)
                        .unwrap()
                        .to_string(),
                )
            } else {
                None
            }
        })
        .expect("The argument '--ram <RAM>' requires a value but none was supplied");

    // TODO: wait for autoscaling to be implemented
    // deployment_config.container_strategy = options
    //     .config
    //     .scaling_strategy
    //     .or_else(|| {
    //         if is_update {
    //             Some(deployment_config.container_strategy.clone())
    //         } else {
    //             None
    //         }
    //     })
    //     .expect(
    //         "The argument '--strategy <SCALING_STRATEGY>' requires a value but none was supplied",
    //     );

    // if deployment_config.container_strategy == ScalingStrategy::Autoscaled {
    //     container_options.containers = None;

    //     container_options.min_containers = Some(
    //         options.config.min_containers
    //         .or({
    //             if is_update {
    //                 container_options.min_containers
    //             } else {
    //                 None
    //             }
    //         })
    //         .expect("The argument '--min-containers <MIN_CONTAINERS>' requires a value but none was supplied"),
    //     );

    //     container_options.max_containers = Some(
    //         options.config.max_containers
    //         .or({
    //             if is_update {
    //                 container_options.max_containers
    //             } else {
    //                 None
    //             }
    //         })
    //         .expect("The argument '--max-containers <MAX_CONTAINERS>' requires a value but none was supplied"),
    //     );
    // } else {
    //     container_options.min_containers = None;
    //     container_options.max_containers = None;

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
    // }

    if let Some(env) = options.config.env {
        deployment_config.env.extend(
            env.iter()
                .map(|kv| (kv.0.clone(), kv.1.clone()))
                .collect::<Vec<(String, String)>>(),
        );
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
    let name = fallback_name
        .clone()
        .or_else(|| deployment_config.name.clone())
        .unwrap_or_default();

    let name = dialoguer::Input::<String>::new()
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

    if !is_update || deployment_config.name != Some(name.clone()) {
        deployment_config.name = Some(name);
    } else {
        deployment_config.name = None;
    }

    log::debug!("Deployment name: {:?}", deployment_config.name);

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

    deployment_config.type_ = Some(ask_question_iter(
        "Container type",
        &ContainerType::values(),
        deployment_config.type_.clone(),
    )?);

    if deployment_config.type_ != Some(ContainerType::Ephemeral) {
        deployment_config.restart_policy = Some(ask_question_iter(
            "Restart policy",
            &RestartPolicy::values(),
            deployment_config.restart_policy.clone(),
        )?);
    }

    // TODO: wait for autoscaling to be implemented
    // deployment_config.container_strategy = ask_question_iter(
    //     "Scaling strategy",
    //     &ScalingStrategy::values(),
    //     Some(deployment_config.container_strategy.clone()),
    // )?;
    deployment_config.container_strategy = ScalingStrategy::Manual;

    deployment_config.resources.vcpu = dialoguer::Input::<f64>::new()
        .with_prompt("CPUs")
        .default(deployment_config.resources.vcpu)
        .validate_with(validate_cpu_count)
        .interact_text()?;

    deployment_config.resources.ram = ask_question_iter(
        "RAM",
        &RamSizes::values(),
        RamSizes::from_str(&deployment_config.resources.ram).ok(),
    )?
    .to_string();

    // if deployment_config.container_strategy == ScalingStrategy::Autoscaled {
    //     container_options.containers = None;

    //     container_options.min_containers = Some(
    //         dialoguer::Input::<u64>::new()
    //             .with_prompt("Minimum container amount")
    //             .default(1)
    //             .validate_with(|containers: &u64| -> Result<(), &str> {
    //                 if *containers > 0 {
    //                     Ok(())
    //                 } else if *containers > 10 {
    //                     Err("Container amount must be less than or equal to 10")
    //                 } else {
    //                     Err("Container amount must be greater than 0")
    //                 }
    //             })
    //             .interact()
    //             .expect("Failed to get minimum containers"),
    //     );
    //     container_options.max_containers = Some(
    //         dialoguer::Input::<u64>::new()
    //             .with_prompt("Maximum container amount")
    //             .default(10)
    //             .validate_with(|containers: &u64| -> Result<(), &str> {
    //                 if *containers > 0 {
    //                     Ok(())
    //                 } else if *containers > 10 {
    //                     Err("Container amount must be less than or equal to 10")
    //                 } else {
    //                     Err("Container amount must be greater than 0")
    //                 }
    //             })
    //             .interact()
    //             .expect("Failed to get maximum containers"),
    //     );
    // } else {
    //     container_options.min_containers = None;
    //     container_options.max_containers = None;

    container_options.containers = Some(
        dialoguer::Input::<u64>::new()
            .with_prompt("Container amount to start")
            .default(1)
            .validate_with(|containers: &u64| -> Result<(), &str> {
                if *containers < 1 {
                    Err("Container amount must be at least 1")
                } else if *containers > 10 {
                    Err("Container amount must be less than or equal to 10")
                } else {
                    Ok(())
                }
            })
            .interact_text()?,
    );
    // }

    deployment_config.env = get_multiple_envs()?;

    Ok((deployment_config.clone(), container_options.clone()))
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
