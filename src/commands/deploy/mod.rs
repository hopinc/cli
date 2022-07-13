mod util;

use std::collections::HashMap;
use std::env::current_dir;
use std::path::PathBuf;

use self::util::compress;
use crate::commands::ignite::types::{
    ContainerType, CreateDeployment, Image, Resources, ScalingStrategy, SingleDeployment,
};
use crate::config::{HOP_BUILD_BASE_URL, HOP_REGISTRY_URL};
use crate::state::State;
use crate::store::hopfile::HopFile;
use crate::{done, info, warn};
use hyper::Method;
use reqwest::multipart::{Form, Part};
use structopt::StructOpt;
use tokio::fs;

#[derive(Debug, StructOpt, Default, PartialEq)]
pub struct DeploymentConfig {
    #[structopt(
        short = "n",
        long = "name",
        help = "Name of the deployment, defaults to the directory name"
    )]
    name: Option<String>,

    #[structopt(
        short = "t",
        long = "type",
        help = "Type of the container, defaults to `ephemeral`"
    )]
    container_type: Option<ContainerType>,

    #[structopt(
        short = "c",
        long = "cpu",
        help = "The number of CPUs to use, defaults to 1"
    )]
    cpu: Option<u64>,

    #[structopt(
        short = "m",
        long = "ram",
        help = "Amount of RAM to use, defaults to 512MB"
    )]
    ram: Option<String>,

    #[structopt(
        short = "e",
        long = "env",
        help = "Environment variables to set, in the form of KEY=VALUE"
    )]
    env: Option<Vec<String>>,

    #[structopt(
        short = "s",
        long = "scaling",
        help = "Scaling strategy, defaults to `manual`"
    )]
    scaling_strategy: Option<ScalingStrategy>,

    #[structopt(
        short = "i",
        long = "containers",
        help = "Number of containers to use, defaults to 1 if `scaling` is manual",
        required_if("scaling", "manual")
    )]
    _containers: Option<u64>,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Deploy a new container")]
pub struct DeployOptions {
    #[structopt(
        name = "dir",
        help = "Directory to deploy, defaults to current directory"
    )]
    path: Option<PathBuf>,

    #[structopt(flatten)]
    config: DeploymentConfig,
}

pub async fn handle_deploy(options: DeployOptions, state: State) -> Result<(), std::io::Error> {
    let mut dir = current_dir().expect("Could not get current directory");

    if let Some(path) = options.path {
        dir = dir
            .join(path)
            .canonicalize()
            .expect("Could not get canonical path");
    }

    info!("Attempting to deploy {}", dir.display());

    let (_hopfile, deployment) = match HopFile::find(dir.clone()).await {
        Some(hopfile) => {
            info!("Found hopfile: {}", hopfile.path.display());

            // TODO: possible update of deployment if it already exists?
            let deployment = state
                .http
                .request::<SingleDeployment>(
                    "GET",
                    format!("/ignite/deployments/{}", hopfile.config.deployment_id).as_str(),
                    None,
                )
                .await
                .expect("Failed to get deployment")
                .unwrap()
                .deployment;

            if options.config != DeploymentConfig::default() {
                warn!("Deployment exists, skipping arguments");
            }

            (hopfile, deployment)
        }

        None => {
            info!("No hopfile found, creating one");

            let project = state.ctx.current_project_error();

            let mut hopfile = HopFile::new(
                dir.clone().join("hop.yml"),
                project.clone().id,
                // override later when created in the API
                String::new(),
            );

            // TODO: run a walkthrough to setup the deployment?
            let name = options
                .config
                .name
                .unwrap_or_else(|| dir.file_name().unwrap().to_str().unwrap().to_string());

            let deployment_config = CreateDeployment {
                container_strategy: options
                    .config
                    .scaling_strategy
                    .unwrap_or(ScalingStrategy::Manual),
                container_type: options
                    .config
                    .container_type
                    .unwrap_or(ContainerType::Persistent),
                name: name.clone(),
                env: options
                    .config
                    .env
                    .map(|env| {
                        env.iter()
                            .map(|env| {
                                let mut split = env.split("=");
                                let key = split.next().unwrap_or("");
                                let value = split.next().unwrap_or("");

                                (key.to_string(), value.to_string())
                            })
                            .collect()
                    })
                    .unwrap_or(HashMap::new()),
                image: Image {
                    name: format!(
                        "{}/{}/{}",
                        HOP_REGISTRY_URL,
                        project.namespace.clone(),
                        name
                    ),
                },
                resources: Resources {
                    cpu: options.config.cpu.unwrap_or(1),
                    ram: options.config.ram.unwrap_or("512M".to_string()),
                    vgpu: vec![],
                },
            };

            let deployment = state
                .http
                .request::<SingleDeployment>(
                    "POST",
                    format!(
                        "/ignite/deployments?project={}",
                        hopfile.config.project_id.clone()
                    )
                    .as_str(),
                    Some((
                        serde_json::to_string(&deployment_config).unwrap().into(),
                        "application/json",
                    )),
                )
                .await
                .expect("Error while creating deployment")
                .unwrap()
                .deployment;

            hopfile.config.deployment_id = deployment.id.clone();

            hopfile
                .clone()
                .save()
                .await
                .expect("Could not save hopfile");

            (hopfile, deployment)
        }
    };

    // deployment id is used not to colide if the user is deploying multiple items
    let packed = compress(deployment.id.clone(), dir)
        .await
        .expect("Could not compress");

    info!("Packed to: {}", packed);

    let bytes = fs::read(packed.clone())
        .await
        .expect("Could not read packed file");
    let multipart = Form::new().part("file", Part::bytes(bytes).file_name("deployment.tar.gz"));

    info!("Uploading...");

    let response = state
        .http
        .client
        .request(
            Method::POST,
            format!(
                "{}/deployments/{}/builds",
                HOP_BUILD_BASE_URL, deployment.id
            )
            .as_str(),
        )
        .header("content_type", "multipart/form-data".to_string())
        .multipart(multipart)
        .send()
        .await
        .expect("Failed to send data to build endpoint");

    state
        .http
        .handle_response::<()>(response)
        .await
        .expect("Failed to handle response");

    info!("Deleting archive...");
    fs::remove_file(packed).await?;

    done!("Pushed deployment `{}`", deployment.name);

    Ok(())
}
