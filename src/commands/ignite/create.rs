use clap::Parser;
use console::style;

use super::types::{Env, RamSizes, ScalingStrategy};
use crate::commands::containers::types::ContainerType;
use crate::commands::containers::utils::create_containers;
use crate::commands::ignite::util::{create_deployment, create_deployment_config};
use crate::state::State;

pub const WEB_DEPLOYMENTS_URL: &str = "https://console.hop.io/ignite/deployment/";

#[derive(Debug, Parser, Default, PartialEq, Clone)]
pub struct DeploymentConfig {
    #[clap(short = 'n', long = "name", help = "Name of the deployment")]
    pub name: Option<String>,

    #[clap(
        short = 't',
        long = "type",
        help = "Type of the container, defaults to `persistent`"
    )]
    pub container_type: Option<ContainerType>,

    #[clap(
        short = 's',
        long = "strategy",
        help = "Scaling strategy, defaults to `autoscaled`"
    )]
    pub scaling_strategy: Option<ScalingStrategy>,

    #[clap(
        short = 'c',
        long = "cpu",
        help = "The number of CPUs to use between 1 to 32, defaults to 1"
    )]
    pub cpu: Option<f64>,

    #[clap(
        short = 'r',
        long = "ram",
        help = "Amount of RAM to use between 128MB to 64GB, defaults to 512MB"
    )]
    pub ram: Option<RamSizes>,

    #[clap(
        short = 'd',
        long = "containers",
        help = "Number of containers to deploy if `scaling` is manual, defaults to 1"
    )]
    pub containers: Option<u64>,

    #[clap(
        long = "min-containers",
        help = "Minimum number of containers to use if `scaling` is autoscale, defaults to 1"
    )]
    pub min_containers: Option<u64>,

    #[clap(
        long = "max-containers",
        help = "Maximum number of containers to use if `scaling` is autoscale, defaults to 10"
    )]
    pub max_containers: Option<u64>,

    #[clap(
        short = 'e',
        long = "env",
        help = "Environment variables to set, in the form of `key=value`",
        min_values = 0
    )]
    pub env: Option<Vec<Env>>,
}

#[derive(Debug, Parser, Default, PartialEq, Clone)]
#[clap(about = "Create a new deployment")]
pub struct Options {
    #[clap(flatten)]
    pub config: DeploymentConfig,

    #[clap(short = 'i', long = "image", help = "Image url")]
    pub image: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<(), std::io::Error> {
    let project = state.ctx.current_project_error();

    log::info!(
        "Deploying to project {} /{} ({})",
        project.name,
        project.namespace,
        project.id
    );

    let is_not_guided = options != Options::default();

    let (deployment_config, container_options) =
        create_deployment_config(options, is_not_guided, &None);

    let deployment = create_deployment(state.http.clone(), project.id, deployment_config).await;

    log::info!(
        "Deployment `{}` ({}) created",
        deployment.name,
        deployment.id
    );

    if let Some(count) = container_options.containers {
        if count > 0 {
            log::info!("Creating {} containers", count);
            create_containers(state.http, deployment.id.clone(), count).await;
        }
    }

    log::info!(
        "Deployed successfuly, you can find it at: {}",
        style(format!(
            "{}{}?project={}",
            WEB_DEPLOYMENTS_URL, deployment.id, project.namespace
        ))
        .underlined()
        .bold()
    );

    Ok(())
}
