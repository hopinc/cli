use anyhow::Result;
use clap::Parser;

use super::types::{Env, RestartPolicy, ScalingStrategy, VolumeFs};
use crate::commands::containers::utils::create_containers;
use crate::commands::ignite::types::Deployment;
use crate::commands::ignite::utils::{create_deployment, update_deployment_config, WEB_IGNITE_URL};
use crate::state::State;
use crate::utils::urlify;

#[derive(Debug, Parser, Default, PartialEq, Clone)]
pub struct DeploymentConfig {
    #[clap(short = 'n', long = "name", help = "Name of the deployment")]
    pub name: Option<String>,

    #[clap(
        short = 's',
        long = "strategy",
        help = "Scaling strategy, defaults to `autoscale`"
    )]
    pub scaling_strategy: Option<ScalingStrategy>,

    #[clap(short, long, help = "Tier of the deployment")]
    pub tier: Option<String>,

    #[clap(short, long, help = "The number of CPUs to use, overrides the tier")]
    pub cpu: Option<f64>,

    #[clap(short = 'm', long, help = "Amount of RAM to use, overrides the tier")]
    pub ram: Option<String>,

    #[clap(
        short = 'd',
        long,
        help = "Amount of containers to deploy if `scaling` is manual, defaults to 1"
    )]
    pub containers: Option<u64>,

    #[clap(
        short,
        long,
        help = "Environment variables to set, in the form of `key=value`"
    )]
    pub env: Option<Vec<Env>>,

    #[clap(
        short,
        long = "restart-policy",
        help = "Restart policy, defaults to `on-failure`"
    )]
    pub restart_policy: Option<RestartPolicy>,

    #[clap(flatten)]
    pub volume: VolumeConfig,

    #[clap(long, help = "Entrypoint to use")]
    pub entrypoint: Option<String>,

    #[clap(long, help = "Command to use")]
    pub command: Option<String>,

    #[clap(long, help = "Make containers delete on exit")]
    pub rm: bool,
}

#[derive(Debug, Parser, Default, PartialEq, Eq, Clone)]
pub struct VolumeConfig {
    #[clap(short, long, help = "Volume mount to use")]
    pub volume_mount: Option<String>,

    #[clap(long, help = "Size of the volume to use, defaults to 3GB")]
    pub volume_size: Option<String>,

    #[clap(long, help = "Type of the volume file system, defaults to `ext4`")]
    pub volume_fs: Option<VolumeFs>,
}

#[derive(Debug, Parser, Default, PartialEq, Clone)]
#[clap(about = "Create a new deployment")]
pub struct Options {
    #[clap(flatten)]
    pub config: DeploymentConfig,

    #[clap(short = 'i', long = "image", help = "Image url")]
    pub image: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project = state.ctx.current_project_error()?;

    log::info!(
        "Deploying to project {} /{} ({})",
        project.name,
        project.namespace,
        project.id
    );

    let is_visual = options == Options::default();

    let (deployment_config, container_options) = update_deployment_config(
        &state.http,
        options,
        is_visual,
        &Deployment::default(),
        &None,
        false,
    )
    .await?;

    let deployment = create_deployment(&state.http, &project.id, &deployment_config).await?;

    log::info!(
        "Deployment `{}` ({}) created",
        deployment.name,
        deployment.id
    );

    if let Some(count) = container_options.containers {
        if count > 0 {
            log::info!("Creating {} containers", count);

            create_containers(&state.http, &deployment.id, count).await?;
        }
    }

    log::info!(
        "Deployed successfully, you can find it at: {}",
        urlify(&format!(
            "{}{}?project={}",
            WEB_IGNITE_URL, deployment.id, project.namespace
        ))
    );

    Ok(())
}
