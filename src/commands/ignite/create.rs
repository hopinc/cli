use clap::Parser;

use super::types::{
    ContainerType, CreateDeployment, Deployment, RamSizes, ScalingStrategy, SingleDeployment,
};
use crate::commands::deploy::types::{CreateContainers, Env};
use crate::commands::deploy::util::create_deployment_config;
use crate::state::http::HttpClient;
use crate::state::State;

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
    pub cpu: Option<u64>,

    #[clap(
        short = 'r',
        long = "ram",
        help = "Amount of RAM to use between 128MB to 64GB, defaults to 512MB"
    )]
    pub ram: Option<RamSizes>,

    #[clap(
        short = 'd',
        long = "containers",
        help = "Number of containers to use if `scaling` is manual, defaults to 1"
    )]
    pub containers: Option<u64>,

    #[clap(
        long = "min",
        help = "Minimum number of containers to use if `scaling` is autoscaled, defaults to 1"
    )]
    pub min_containers: Option<u64>,

    #[clap(
        long = "max",
        help = "Maximum number of containers to use if `scaling` is autoscaled, defaults to 10"
    )]
    pub max_containers: Option<u64>,

    #[clap(
        short = 'e',
        long = "env",
        help = "Environment variables to set, in the form of `key=value`"
    )]
    pub env: Option<Vec<Env>>,
}

#[derive(Debug, Parser)]
pub struct CreateOptions {
    #[clap(flatten)]
    config: DeploymentConfig,

    #[clap(short = 'i', long = "image", help = "Image url")]
    pub image: Option<String>,
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

pub async fn handle_create(options: CreateOptions, state: State) -> Result<(), std::io::Error> {
    let project = state.ctx.current_project_error();

    log::info!(
        "Deploying to project {} /{} ({})",
        project.name,
        project.namespace,
        project.id
    );

    let is_not_quided = options.config != DeploymentConfig::default();

    let (mut deployment_config, container_options) =
        create_deployment_config(options.config, is_not_quided, None).await;

    let image = if is_not_quided {
        dialoguer::Input::<String>::new()
            .with_prompt("Image url")
            .interact()
            .expect("Could not get image url")
    } else {
        options
            .image
            .expect("The argument '--image <IMAGE>' requires a value but none was supplied")
    };

    deployment_config.image.name = image;

    let deployment = create_deployment(state.http.clone(), project.id, deployment_config).await;

    log::info!(
        "Deployment `{}` ({}) created",
        deployment.name,
        deployment.id
    );

    let create_containers = CreateContainers {
        count: container_options
            .containers
            .expect("type check: no container count"),
    };

    state
        .http
        .request::<()>(
            "POST",
            format!("/ignite/deployments/{}/containers", deployment.id).as_str(),
            Some((
                serde_json::to_string(&create_containers).unwrap().into(),
                "application/json",
            )),
        )
        .await
        .expect("Failed to create containers");

    log::info!("Deployment created successfully");

    Ok(())
}
