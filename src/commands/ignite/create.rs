use clap::Parser;

use crate::{
    commands::deploy::{types::Env, util::create_deployment_config},
    done, info,
    state::{http::HttpClient, State},
};

use super::types::{
    ContainerType, CreateDeployment, Deployment, RamSizes, ScalingStrategy, SingleDeployment,
};

#[derive(Debug, Parser, Default, PartialEq)]
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
        long = "scaling",
        help = "Scaling strategy, defaults to `autoscaled`"
    )]
    pub scaling_strategy: Option<ScalingStrategy>,

    #[clap(
        short = 'c',
        long = "cpu",
        help = "The number of CPUs to use, defaults to 1"
    )]
    pub cpu: Option<u64>,

    #[clap(
        short = 'r',
        long = "ram",
        help = "Amount of RAM to use, defaults to 512MB"
    )]
    pub ram: Option<RamSizes>,

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

    info!(
        "Deploying to project {} /{} ({})",
        project.name, project.namespace, project.id
    );

    let is_quiet = options.config != DeploymentConfig::default();

    let mut config = create_deployment_config(options.config, None).await;

    let image = if is_quiet {
        options
            .image
            .expect("The argument '--image <IMAGE>' requires a value but none was supplied")
    } else {
        dialoguer::Input::<String>::new()
            .with_prompt("Image url")
            .interact()
            .expect("Could not get image url")
    };

    config.image.name = image;

    let deployment = create_deployment(state.http, project.id, config).await;

    done!(
        "Deployment `{}` ({}) created",
        deployment.name,
        deployment.id
    );

    Ok(())
}
