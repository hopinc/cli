mod builder;
mod local;
pub mod util;

use std::env::current_dir;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use self::util::env_file_to_map;
use crate::commands::containers::types::ContainerOptions;
use crate::commands::containers::utils::create_containers;
use crate::commands::gateways::create::GatewayOptions;
use crate::commands::gateways::types::{GatewayConfig, GatewayType};
use crate::commands::gateways::util::{create_gateway, update_gateway_config};
use crate::commands::ignite::create::{
    DeploymentConfig, Options as CreateOptions, WEB_DEPLOYMENTS_URL,
};
use crate::commands::ignite::types::{
    CreateDeployment, Deployment, ScalingStrategy, SingleDeployment,
};
use crate::commands::ignite::util::{create_deployment, rollout, update_deployment_config};
use crate::commands::projects::util::format_project;
use crate::state::State;
use crate::store::hopfile::HopFile;
use crate::util::urlify;

const HOP_BUILD_BASE_URL: &str = "https://builder.hop.io/v1";
const HOP_REGISTRY_URL: &str = "registry.hop.io";

#[derive(Debug, Parser)]
#[clap(about = "Deploy a new container")]
pub struct Options {
    #[clap(
        name = "dir",
        help = "Directory to deploy, defaults to current directory"
    )]
    path: Option<PathBuf>,

    #[clap(flatten)]
    config: DeploymentConfig,

    #[clap(
        short = 'E',
        long = "env-file",
        help = "Load environment variables from a .env file in the current directory, in the form of KEY=VALUE"
    )]
    envfile: bool,

    #[clap(
        short = 'y',
        long = "yes",
        help = "Use the default yes answer to all prompts"
    )]
    yes: bool,

    #[clap(
        short = 'l',
        long = "local",
        help = "Build the container locally using nixpacks or docker instead of using the builder"
    )]
    local: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let mut dir = current_dir().expect("Could not get current directory");

    if let Some(path) = options.path {
        dir = dir
            .join(path)
            .canonicalize()
            .expect("Could not get canonical path");
    }

    assert!(dir.is_dir(), "{} is not a directory", dir.display());

    log::info!("Attempting to deploy {}", dir.display());

    let is_not_guided = options.config != DeploymentConfig::default();

    let (project, deployment, container_options, existing) = match HopFile::find(dir.clone()).await
    {
        Some(hopfile) => {
            log::info!("Found hopfile: {}", hopfile.path.display());

            // TODO: possible update of deployment if it already exists?
            let deployment = state
                .http
                .request::<SingleDeployment>(
                    "GET",
                    &format!("/ignite/deployments/{}", hopfile.config.deployment_id),
                    None,
                )
                .await
                .expect("Failed to get deployment")
                .unwrap()
                .deployment;

            // if deployment exists it's safe to unwrap
            let project = state
                .ctx
                .find_project_by_id_or_namespace(hopfile.config.project_id)
                .unwrap();

            if is_not_guided {
                log::warn!("Deployment exists, skipping arguments");
            }

            log::info!("Deploying to project {}", format_project(&project));

            // TODO: update when autoscaling is supported
            let container_options = ContainerOptions {
                containers: Some(deployment.container_count),
                min_containers: None,
                max_containers: None,
            };

            (project, deployment, container_options, true)
        }

        None => {
            log::info!("No hopfile found, creating one");

            let project = state.ctx.clone().current_project_error();

            log::info!("Deploying to project {}", format_project(&project));

            let default_name = dir
                .clone()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                // make the filename semi safe
                .replace('_', "-")
                .replace(' ', "-")
                .replace('.', "-")
                .to_lowercase();

            let (mut deployment_config, container_options) = if options.yes {
                log::warn!("Using default config, skipping arguments");

                (
                    CreateDeployment {
                        name: Some(default_name),
                        // TODO: remove after autoscaling is supported
                        container_strategy: ScalingStrategy::Manual,
                        ..Default::default()
                    },
                    ContainerOptions {
                        containers: Some(1),
                        min_containers: None,
                        max_containers: None,
                    },
                )
            } else {
                update_deployment_config(
                    CreateOptions {
                        config: options.config.clone(),
                        // temporary value that gets replaced after we get the name
                        image: Some("".to_string()),
                    },
                    is_not_guided,
                    &Deployment::default(),
                    &Some(default_name),
                )?
            };

            deployment_config.image.name = format!(
                "{}/{}/{}",
                HOP_REGISTRY_URL,
                project.namespace,
                deployment_config.name.clone().unwrap()
            );

            if options.envfile {
                deployment_config
                    .env
                    .extend(env_file_to_map(dir.join(".env")).await);
            }

            let deployment =
                create_deployment(&state.http, &project.id, &deployment_config).await?;

            // skip gateway creation if using default config
            if !options.yes
                && !is_not_guided
                && dialoguer::Confirm::new()
                    .with_prompt("Do you want to create a Gateway? (You can always add one later)")
                    .interact()?
            {
                let gateway_config = update_gateway_config(
                    &GatewayOptions::default(),
                    false,
                    &GatewayConfig::default(),
                )?;

                let gateway = create_gateway(&state.http, &deployment.id, &gateway_config).await?;

                log::info!("Created Gateway `{}`", gateway.id);

                if gateway.type_ == GatewayType::External {
                    log::info!(
                        "Your deployment will be accesible via {}",
                        urlify(&gateway.full_url())
                    );
                }
            }

            HopFile::new(
                dir.clone().join("hop.yml"),
                project.id.clone(),
                deployment.id.clone(),
            )
            .save()
            .await?;

            (project, deployment, container_options, false)
        }
    };

    if !options.local {
        builder::build(&state, &project.id, &deployment.id, dir.clone()).await?;
    } else {
        local::build(&state, &deployment.config.image.name, dir.clone()).await?;
    }

    if existing {
        if deployment.container_count > 0 {
            log::info!("Rolling out new containers");
            rollout(&state.http, &deployment.id).await?;
        }
    } else if let Some(containers) = container_options
        .containers
        .or(container_options.min_containers)
    {
        create_containers(&state.http, &deployment.id, containers).await?;
    }

    log::info!(
        "Deployed successfuly, you can find it at: {}",
        urlify(&format!(
            "{}{}?project={}",
            WEB_DEPLOYMENTS_URL, deployment.id, project.namespace
        ))
    );

    Ok(())
}
