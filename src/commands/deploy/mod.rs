pub mod types;
pub mod util;

use std::env::current_dir;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use console::style;
use hyper::Method;
use reqwest::multipart::{Form, Part};
use tokio::fs;

use self::types::{Event, Message};
use self::util::{compress, env_file_to_map};
use crate::commands::containers::types::ContainerOptions;
use crate::commands::containers::utils::create_containers;
use crate::commands::ignite::create::{
    DeploymentConfig, Options as CreateOptions, WEB_DEPLOYMENTS_URL,
};
use crate::commands::ignite::types::SingleDeployment;
use crate::commands::ignite::util::{create_deployment, create_deployment_config, rollout};
use crate::state::State;
use crate::store::hopfile::HopFile;

const HOP_BUILD_BASE_URL: &str = "https://builder.hop.io/v1";
const HOP_REGISTRY_URL: &str = "registry.hop.io";

#[derive(Debug, Parser)]
#[structopt(about = "Deploy a new container")]
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
}

#[allow(clippy::too_many_lines)]
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
                    format!("/ignite/deployments/{}", hopfile.config.deployment_id).as_str(),
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

            log::info!(
                "Deploying to project {} /{} ({})",
                project.name,
                project.namespace,
                project.id
            );

            // TODO: update when autoscaling is supported
            let container_options = ContainerOptions {
                containers: Some(deployment.container_count.into()),
                min_containers: None,
                max_containers: None,
            };

            (project, deployment, container_options, true)
        }

        None => {
            log::info!("No hopfile found, creating one");

            let project = state.ctx.current_project_error();

            log::info!(
                "Deploying to project {} /{} ({})",
                project.name,
                project.namespace,
                project.id
            );

            let mut hopfile = HopFile::new(
                dir.clone().join("hop.yml"),
                project.clone().id,
                // override later when created in the API
                String::new(),
            );

            let (mut deployment_config, container_options) = create_deployment_config(
                CreateOptions {
                    config: options.config.clone(),
                    // temporary value that gets replaced after we get the name
                    image: Some("temp".to_string()),
                },
                is_not_guided,
                &Some(
                    dir.clone()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                ),
            );

            deployment_config.image.name = format!(
                "{}/{}/{}",
                HOP_REGISTRY_URL, project.namespace, deployment_config.name
            );

            if options.envfile {
                deployment_config
                    .env
                    .extend(env_file_to_map(dir.join(".env")).await);
            }

            let deployment = create_deployment(
                state.http.clone(),
                hopfile.config.project_id.clone(),
                deployment_config,
            )
            .await;

            hopfile.config.deployment_id = deployment.id.clone();

            hopfile
                .clone()
                .save()
                .await
                .expect("Could not save hopfile");

            (project, deployment, container_options, false)
        }
    };

    // connect to leap here so no logs interfere with the deploy
    let mut connection = state
        .ws
        .connect()
        .await
        .expect("Could not connect to Leap Edge");

    // deployment id is used not to colide if the user is deploying multiple items
    let packed = compress(deployment.id.clone(), dir)
        .await
        .expect("Could not compress");

    log::info!("Packed to: {}", packed);

    let bytes = fs::read(packed.clone())
        .await
        .expect("Could not read packed file");

    let multipart = Form::new().part(
        "file",
        Part::bytes(bytes)
            .file_name("deployment.tar.gz")
            .mime_str("application/x-gzip")
            .unwrap(),
    );

    log::info!("Uploading...");

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

    log::info!("Deleting archive...");
    fs::remove_file(packed).await?;

    log::info!("From Hop builder:");

    while let Some(data) = connection.recieve_message::<Message>().await {
        // build logs are sent only in DMs
        if data.e != "DIRECT_MESSAGE" {
            continue;
        }

        let build_event: Event = serde_json::from_value(data.d).unwrap();

        let build_data = if build_event.d.is_none() {
            continue;
        } else {
            build_event.d.unwrap()
        };

        match build_event.e.as_str() {
            "BUILD_PROGRESS" => {
                if let Some(progress) = build_data.progress {
                    print!("{}", progress);
                }
            }

            "PUSH_SUCCESS" => {
                connection.close().await;
                println!();
                log::info!("Pushed successfully");
                break;
            }

            "PUSH_FAILURE" => {
                connection.close().await;
                println!();
                panic!(
                    "Push failed, for help contact us on {} and mention the deployment id: {}",
                    style("https://discord.gg/hop").underlined().bold(),
                    deployment.id
                );
            }

            // ignore rest
            _ => {}
        }
    }

    if existing {
        if deployment.container_count > 0 {
            log::info!("Rolling out new containers");
            rollout(state.http, deployment.id.clone()).await;
        }
    } else if let Some(containers) = container_options.containers {
        create_containers(state.http, deployment.id.clone(), containers).await;
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
