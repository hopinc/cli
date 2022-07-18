pub mod types;
pub mod util;

use std::env::current_dir;
use std::path::PathBuf;

use clap::Parser;
use hyper::Method;
use reqwest::multipart::{Form, Part};

use tokio::fs;

use self::util::compress;
use super::ignite::create::DeploymentConfig;
use crate::commands::deploy::types::{Data, Message};
use crate::commands::deploy::util::{create_deployment_config, env_file_to_map};
use crate::commands::ignite::create::create_deployment;
use crate::commands::ignite::types::SingleDeployment;
use crate::config::{HOP_BUILD_BASE_URL, HOP_REGISTRY_URL};
use crate::state::State;
use crate::store::hopfile::HopFile;
use crate::{done, info, warn};

#[derive(Debug, Parser)]
#[structopt(about = "Deploy a new container")]
pub struct DeployOptions {
    #[clap(
        name = "dir",
        help = "Directory to deploy, defaults to current directory"
    )]
    path: Option<PathBuf>,

    #[clap(flatten)]
    config: DeploymentConfig,

    #[clap(
        short = 'i',
        long = "containers",
        help = "Number of containers to use, defaults to 1 if `scaling` is manual"
    )]
    containers: Option<u64>,

    #[clap(
        short = 'E',
        long = "env-file",
        help = "Load environment variables from a .env file in the current directory, in the form of KEY=VALUE"
    )]
    pub envfile: bool,
}

pub async fn handle_deploy(options: DeployOptions, state: State) -> Result<(), std::io::Error> {
    let mut dir = current_dir().expect("Could not get current directory");

    if let Some(path) = options.path {
        dir = dir
            .join(path)
            .canonicalize()
            .expect("Could not get canonical path");
    }

    if !dir.is_dir() {
        panic!("{} is not a directory", dir.display());
    }

    let mut connection = state
        .ws
        .connect(state.ctx.me.clone().unwrap().leap_token.as_str())
        .await
        .expect("Could not connect to Leap Edge");

    info!("Attempting to deploy {}", dir.display());

    let deployment = match HopFile::find(dir.clone()).await {
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

            // if deployment exists it's safe to unwrap
            let project = state
                .ctx
                .find_project_by_id_or_namespace(hopfile.config.project_id)
                .unwrap();

            if options.config != DeploymentConfig::default() {
                warn!("Deployment exists, skipping arguments");
            }

            info!(
                "Deploying to project {} /{} ({})",
                project.name, project.namespace, project.id
            );

            deployment
        }

        None => {
            info!("No hopfile found, creating one");

            let project = state.ctx.current_project_error();

            info!(
                "Deploying to project {} /{} ({})",
                project.name, project.namespace, project.id
            );

            let mut hopfile = HopFile::new(
                dir.clone().join("hop.yml"),
                project.clone().id,
                // override later when created in the API
                String::new(),
            );

            let mut config = create_deployment_config(
                options.config,
                Some(
                    dir.clone()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                ),
            )
            .await;

            config.image.name =
                format!("{}/{}/{}", HOP_REGISTRY_URL, project.namespace, config.name);

            if options.envfile {
                config
                    .env
                    .extend(env_file_to_map(dir.join("env.yml")).await);
            }

            let deployment = create_deployment(
                state.http.clone(),
                hopfile.config.project_id.clone(),
                config,
            )
            .await;

            hopfile.config.deployment_id = deployment.id.clone();

            hopfile
                .clone()
                .save()
                .await
                .expect("Could not save hopfile");

            deployment
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
    let multipart = Form::new().part(
        "file",
        Part::bytes(bytes)
            .file_name("deployment.tar.gz")
            .mime_str("application/x-gzip")
            .unwrap(),
    );

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

    info!("From Hop builder:");

    while let Some(data) = connection.recieve_message::<Message>().await {
        // build logs are sent only in DMs
        if data.e != "DIRECT_MESSAGE" {
            continue;
        }

        let data: Data = serde_json::from_value(data.d).unwrap();

        if let Some(data) = data.d {
            print!("{}", data);
        }

        match data.e.as_str() {
            "PUSH_SUCCESS" => {
                connection.close().await;
                println!("");
                info!("Pushed successfully");
                break;
            }

            "PUSH_FAILURE" => {
                connection.close().await;
                println!("");
                panic!("Push failed, for help contact us on https://discord.gg/hop and mention the deployment id: {}", deployment.id);
            }

            // ignore rest
            _ => {}
        }
    }

    done!("Pushed deployment `{}`", deployment.name);

    // TODO: ask to deploy containers

    Ok(())
}
