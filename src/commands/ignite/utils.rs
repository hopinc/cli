use std::collections::hash_map::HashMap;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{anyhow, bail, ensure, Context, Result};
use console::Term;
use regex::Regex;
use serde_json::Value;
use tabwriter::TabWriter;
use tokio::fs;

use super::types::{
    CreateDeployment, Deployment, MultipleDeployments, Premade, Premades, RolloutEvent,
    ScaleRequest, SingleDeployment, Storage, Tier, Tiers,
};
use crate::commands::containers::types::{ContainerOptions, ContainerType};
use crate::commands::ignite::create::Options;
use crate::commands::ignite::types::{
    Image, RamSizes, Resources, RestartPolicy, RolloutResponse, ScalingStrategy, VolumeFs,
};
use crate::commands::projects::types::{Project, Sku};
use crate::commands::projects::utils::{get_quotas, get_skus};
use crate::state::http::HttpClient;
use crate::utils::size::{parse_size, unit_multiplier};
use crate::utils::{ask_question_iter, parse_key_val};

pub const WEB_IGNITE_URL: &str = "https://console.hop.io/ignite";

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
        .request::<SingleDeployment>("GET", &format!("/ignite/deployments/{deployment_id}"), None)
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

pub async fn rollout(http: &HttpClient, deployment_id: &str) -> Result<RolloutEvent> {
    let response = http
        .request::<RolloutResponse>(
            "POST",
            &format!("/ignite/deployments/{deployment_id}/rollouts"),
            None,
        )
        .await?
        .ok_or_else(|| anyhow!("Failed to parse response"))?
        .rollout;

    Ok(response)
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

    Ok(response.premade)
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
    project: &Project,
) -> Result<(CreateDeployment, ContainerOptions)> {
    let mut config = CreateDeployment::from(deployment.clone());
    let mut container_options = ContainerOptions::from_deployment(deployment);
    // make the filename semi safe
    let fallback_name = fallback_name
        .clone()
        .map(|s| s.replace(['_', ' ', '.'], "-").to_lowercase());

    let configs = if is_visual {
        update_config_visual(
            http,
            options,
            &mut config,
            &mut container_options,
            &fallback_name,
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
    }?;

    let (skus, quota) = tokio::join!(get_skus(http), get_quotas(http, &project.id));
    let (skus, quota) = (skus?, quota?);

    quota.can_deploy(&config.resources, &config.volume, project)?;

    let (resources, volume_size) = if !project.is_personal() {
        (config.resources, config.volume.map(|v| v.size))
    } else {
        let (applied, billabe) = quota.get_free_tier_billable(&config.resources, &config.volume)?;

        if applied {
            log::warn!("Free tier has been applied to the resources");
        }

        billabe
    };

    let price = get_price_estimate(&skus, &resources, &volume_size)?;

    log::info!(
        "Estimated monthly price{}: {price}$",
        if config.type_ == Some(ContainerType::Stateful) {
            ""
        } else {
            " per container"
        }
    );

    if is_visual
        && !dialoguer::Confirm::new()
            .with_prompt("Do you want to continue?")
            .interact_opt()?
            .unwrap_or(false)
    {
        bail!("User aborted");
    }

    Ok(configs)
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
        .context("The argument '--name <NAME>' requires a value but none was supplied")?
        .to_lowercase();

    validate_deployment_name(&name)?;

    if !is_update || deployment_config.name != Some(name.clone()) {
        deployment_config.name = Some(name);
    }

    deployment_config.image = Some(
        options
            .image
            .map(|i| Image { name: i })
            .or_else(|| {
                if is_update {
                    Some(deployment_config.image.clone().unwrap_or_default())
                } else {
                    None
                }
            })
            .context("Please specify an image via the `--image` flag")?,
    );

    let tiers = get_tiers(http).await?;

    deployment_config.resources = {
        let mut resources = Resources::default();

        if let Some(tier) = options.config.tier {
            let tier = tiers
                .iter()
                .find(|t| t.name.to_lowercase() == tier.to_lowercase())
                .with_context(|| {
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

    if deployment_config.type_ != Some(ContainerType::Stateful) {
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
    }

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
        deployment_config.entrypoint = Some(get_shell_array(&entry));
    }

    if let Some(cmd) = options.config.command {
        deployment_config.command = Some(get_shell_array(&cmd));
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
            .validate_with(|name: &String| -> Result<()> { validate_deployment_name(name) })
            .interact_text()?
            .trim()
            .to_string()
    };

    if !is_update || deployment_config.name != Some(name.clone()) {
        deployment_config.name = Some(name);
    } else {
        deployment_config.name = None;
    }

    deployment_config.image =
        // if name is "" it's using hopdeploy ie. image is created on the fly
        if options.image.is_some() && options.image.clone().unwrap() == ""  {
            Some(Image {
            ..Default::default()
            })
        } else {
            let old_name = deployment_config.image.clone().unwrap_or_default().name;

            let new_name = dialoguer::Input::<String>::new()
            .with_prompt("Image name")
            .default(old_name.clone())
            .show_default(!old_name.is_empty())
            .validate_with(|image: &String| -> Result<(), &str> {
                if image.is_empty() {
                    Err("Please specify an image")
                } else {
                    Ok(())
                }
            })
            .interact_text()?;

            if old_name != new_name {
                Some(Image {
                    name: new_name,
                })
            } else {
                None
            }
        };

    let mut tiers = get_tiers(http).await?;

    // add custom tier for users to specify their own resources
    tiers.push(Tier {
        name: "Custom".to_string(),
        description: "Customize the tier to your needs".to_string(),
        ..Default::default()
    });

    if is_update {
        let mut tmp = vec![Tier {
            name: "Current".to_string(),
            description: "Do not update the tier".to_string(),
            ..Default::default()
        }];
        tmp.extend(tiers);
        tiers = tmp;
    }

    deployment_config.resources = {
        let idx = dialoguer::Select::new()
            .with_prompt("Select a tier that will suit you well")
            .default(0)
            .items(&tiers.iter().map(|t| t.to_string()).collect::<Vec<String>>())
            .interact()?;

        // first in update is `Current` which is not a tier
        if idx == 0 && is_update {
            // dont update the tier
            deployment_config.resources.clone()
        } else if idx == tiers.len() - 1 {
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

    log::debug!("Deployment type: {:?}", deployment_config.volume);

    // volume only will be some and is_update to false in the `from-compose` command
    if (!is_update && deployment_config.volume.is_some())
        || (!(is_update && deployment_config.image.is_some())
            && dialoguer::Confirm::new()
                .with_prompt("Would you like to attach a volume?")
                .default(false)
                .interact()?)
    {
        // remove the previous line for the question
        Term::stderr().clear_last_lines(1)?;

        deployment_config.type_ = Some(ContainerType::Stateful);

        let mut volume = deployment_config.volume.clone().unwrap_or_default();

        volume.size = dialoguer::Input::<String>::new()
            .with_prompt("Volume size")
            .default(volume.size)
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

    if deployment_config.type_ != Some(ContainerType::Stateful) {
        container_options.containers = Some(
            dialoguer::Input::<u64>::new()
                .with_prompt("Container amount to start")
                .default(container_options.containers.unwrap_or(1))
                .validate_with(|containers: &u64| -> Result<(), &str> {
                    if deployment_config.type_ == Some(ContainerType::Stateful) && *containers > 1 {
                        Err("Stateful deployments can only have 1 container")
                    } else if *containers > 10 {
                        Err("Container amount must be less than or equal to 10")
                    } else {
                        Ok(())
                    }
                })
                .interact_text()?,
        );
    }

    deployment_config.env.extend(get_multiple_envs()?);

    if dialoguer::Confirm::new()
        .with_prompt("Do you want to change advanced settings?")
        .default(false)
        .interact_opt()?
        .unwrap_or(false)
    {
        match deployment_config.type_ {
            Some(ContainerType::Persistent) => {
                if dialoguer::Confirm::new()
                    .with_prompt("Would you like your containers to be deleted when they exit?")
                    .default(false)
                    .interact()?
                {
                    deployment_config.type_ = Some(ContainerType::Ephemeral);
                }
            }

            Some(ContainerType::Ephemeral) => {
                if dialoguer::Confirm::new()
                    .with_prompt("Would you like your containers to be persisted when they exit?")
                    .default(false)
                    .interact()?
                {
                    deployment_config.type_ = Some(ContainerType::Persistent);
                }
            }

            _ => {}
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
                    .map(|s| get_shell_array(&s))?,
            );
        }

        if dialoguer::Confirm::new()
            .with_prompt("Do you want to specify a custom command?")
            .default(false)
            .interact()?
        {
            let cmd = deployment_config
                .command
                .clone()
                .unwrap_or_default()
                .join(" ");

            deployment_config.command = Some(
                dialoguer::Input::<String>::new()
                    .with_prompt("Command")
                    .show_default(is_update && !cmd.is_empty())
                    .default(cmd)
                    .interact_text()
                    .map(|s| get_shell_array(&s))?,
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
    }

    if deployment_config.restart_policy.is_none()
        || deployment_config.type_ == Some(ContainerType::Ephemeral)
    {
        deployment_config.restart_policy = Some(RestartPolicy::OnFailure);
    }

    if is_update && deployment_config.type_ == Some(ContainerType::Stateful) {
        deployment_config.type_ = None;
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

        if !dialoguer::Confirm::new()
            .with_prompt("Add another environment variable?")
            .default(false)
            .interact_opt()?
            .unwrap_or(false)
        {
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

fn validate_deployment_name(name: &str) -> Result<()> {
    const MIN_LENGTH: usize = 1;
    const MAX_LENGTH: usize = 20;

    ensure!(
        name.len() <= MAX_LENGTH,
        "Deployment name must be less than {MAX_LENGTH} characters"
    );

    ensure!(
        name.len() >= MIN_LENGTH,
        "Deployment name must be greater than {MIN_LENGTH} characters"
    );

    let regex = Regex::new(r"(?i)^[a-z0-9-]{1,20}$").unwrap();

    ensure!(
        regex.is_match(name),
        "Deployment name must be lowercase alphanumeric characters or hyphens"
    );

    Ok(())
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

pub fn get_shell_array(entrypoint: &str) -> Vec<String> {
    let regex = Regex::new(r#"".*"|[^\s]+"#).unwrap();

    regex
        .find_iter(entrypoint)
        .map(|m| m.as_str().to_string())
        .collect()
}

pub async fn env_file_to_map(path: PathBuf) -> Result<HashMap<String, String>> {
    let mut env = HashMap::new();

    ensure!(
        path.exists(),
        "Could not find .env file at {}",
        path.display()
    );

    let file = fs::read_to_string(path).await?;
    let lines = file.lines();

    for line in lines {
        let line = line.trim();

        // ignore comments
        if line.starts_with('#') {
            continue;
        }

        if line.is_empty() {
            continue;
        }

        match parse_key_val(line) {
            Ok((key, value)) => {
                env.insert(key, value);
            }
            Err(e) => log::warn!("Failed to parse env file line: {}", e),
        }
    }

    Ok(env)
}

pub fn format_premade(premades: &[Premade], title: bool) -> Result<Vec<String>> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(&mut tw, "NAME\tDESCRIPTION")?;
    }

    for premade in premades {
        if title {
            writeln!(&mut tw, "{}\t{}", premade.name, premade.description)?;
        } else {
            writeln!(&mut tw, "{} - {}", premade.name, premade.description)?;
        }
    }

    Ok(String::from_utf8(tw.into_inner()?)?
        .lines()
        .map(std::string::ToString::to_string)
        .collect())
}

const MONTH_IN_MINUTES: f64 = 43200.0;

pub fn get_price_estimate(
    skus: &[Sku],
    resources: &Resources,
    volume: &Option<String>,
) -> Result<String> {
    let mut total = 0.0;

    for sku in skus.iter().filter(|sku| sku.product == "ignite") {
        let mut price = sku.price;

        match sku.id.as_str() {
            "ignite_vcpu_per_min" => {
                price *= resources.vcpu;
            }

            // per 100 MB
            "ignite_ram_per_min" => {
                price *= parse_size(&resources.ram)? as f64;
                price /= (100 * unit_multiplier::MB) as f64;
            }

            // per 1MB
            "ignite_volume_per_min" => {
                if let Some(size) = &volume {
                    price *= parse_size(size)? as f64;
                    price /= unit_multiplier::MB as f64;
                }
            }

            _ => continue,
        }

        total += price;
    }

    total *= MONTH_IN_MINUTES;

    Ok(format!("{total:.2}"))
}

pub async fn get_storage(http: &HttpClient, deployment_id: &str) -> Result<Storage> {
    let data = http
        .request::<Storage>(
            "GET",
            &format!("/ignite/deployments/{deployment_id}/storage",),
            None,
        )
        .await?
        .ok_or_else(|| anyhow!("Failed to parse response"))?;

    Ok(data)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_entrypoint_array() {
        let entrypoint = r#"/bin/bash -c "echo hello world""#;

        let mut entrypoint_array = get_shell_array(entrypoint).into_iter();

        assert_eq!(entrypoint_array.next(), Some("/bin/bash".to_string()));
        assert_eq!(entrypoint_array.next(), Some("-c".to_string()));
        assert_eq!(
            entrypoint_array.next(),
            Some(r#""echo hello world""#.to_string())
        );
        assert_eq!(entrypoint_array.next(), None);
    }

    #[test]
    fn test_price_estimate() {
        let skus = vec![
            Sku {
                id: "ignite_ram_per_min".to_string(),
                product: "ignite".to_string(),
                price: 0.0000060896,
            },
            Sku {
                id: "ignite_vcpu_per_min".to_string(),
                product: "ignite".to_string(),
                price: 0.0001,
            },
            Sku {
                id: "ignite_volume_per_min".to_string(),
                product: "ignite".to_string(),
                price: 0.0000000035,
            },
        ];

        let resources = Resources {
            ram: "1GB".to_string(),
            vcpu: 1.0,
            ..Default::default()
        };

        let volume = Some("1GB".to_string());

        let estimate = get_price_estimate(&skus, &resources, &volume).unwrap();

        assert_eq!(estimate, "7.17");
    }
}
