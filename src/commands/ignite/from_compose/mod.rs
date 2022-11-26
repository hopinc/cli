mod types;
pub mod utils;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;
use regex::bytes::Regex;
use tokio::fs;

use self::types::DockerCompose;
use self::utils::order_by_dependencies;
use crate::commands::auth::docker::HOP_REGISTRY_URL;
use crate::commands::deploy::{builder, local};
use crate::commands::gateways::types::GatewayConfig;
use crate::commands::gateways::util::{create_gateway, update_gateway_config};
use crate::commands::ignite::create::Options as CreateOptions;
use crate::commands::ignite::from_compose::types::ServiceBuildUnion;
use crate::commands::ignite::types::Deployment;
use crate::commands::ignite::utils::{create_deployment, scale, update_deployment_config};
use crate::state::State;
use crate::store::hopfile::HopFile;

#[derive(Debug, Parser)]
#[clap(about = "Creates new Ignite deployments from a Docker compose file")]
pub struct Options {
    #[clap(
        name = "file",
        help = "The file to read from. Defaults to docker-compose.yml"
    )]
    pub file: Option<PathBuf>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let file = match options.file {
        Some(file) => file,
        None => Path::new("docker-compose.yml").to_path_buf(),
    };

    if !file.exists() {
        bail!("File {} does not exist", file.display());
    }

    let parent_dir = file
        .parent()
        .with_context(|| format!("Could not get parent directory of {}", file.display()))?;

    let compose = fs::read(file.clone()).await?;

    let compose: DockerCompose = match serde_yaml::from_slice(&compose) {
        Ok(compose) => compose,
        Err(error) => {
            log::debug!("Failed to parse compose file: {}", error);

            // note from alistair — I am writing this file as I am learning rust. currently I have no idea
            // how I can implement a custom Deserialize that will provide a better error message
            // including the name of the field that failed to deserialize. So, the code below
            // is just parsing the error string.

            // Reading:
            // https://stackoverflow.com/questions/61107467/is-there-a-way-to-extract-the-missing-field-name-from-serde-jsonerror

            let message = error.to_string();

            let captures =
                Regex::new(r"unknown field `(.*)`, expected .* at line (.*) column (.*)");

            if captures.is_err() {
                bail!(
                    "Failed to parse docker-compose.yml: {}",
                    captures.err().unwrap()
                );
            }

            let captures = captures.unwrap().captures(message.as_bytes());

            if captures.is_none() {
                bail!("Failed to parse Docker compose: {error}");
            }

            let captures = captures.unwrap();

            let field = std::str::from_utf8(captures.get(1).unwrap().as_bytes()).unwrap();
            let line = std::str::from_utf8(captures.get(2).unwrap().as_bytes()).unwrap();
            let column = std::str::from_utf8(captures.get(3).unwrap().as_bytes()).unwrap();

            bail!("Failed to parse Docker compose. The Hop CLI does not currently support the `{field}` field at line {line} column {column}" );
        }
    };

    compose.validate()?;

    let project = state.ctx.clone().current_project_error();

    let services = compose.services.unwrap_or_default();
    // let volumes = compose.volumes.unwrap_or_default();

    let mut services = services.iter().collect::<Vec<_>>();

    order_by_dependencies(&mut services);

    log::info!("Creating deployments from {}", file.display());
    log::info!("Found {} services", services.len());

    log::info!("Using project `{}` ({})", project.name, project.namespace);

    let mut deployments_with_gateway = vec![];

    for (name, service) in services {
        log::info!("Creating deployment for {name}");

        let deployment: Deployment = service.clone().into();

        let mut deployment_config = update_deployment_config(
            &state.http,
            CreateOptions {
                config: Default::default(),
                // temporary value that gets replaced after we get the name
                image: if service.build.is_some() {
                    Some("".to_string())
                } else {
                    service.image.clone()
                },
            },
            true,
            &deployment,
            &Some(name.clone()),
            false,
        )
        .await?;

        let dep_name = deployment_config
            .0
            .name
            .clone()
            .unwrap_or_else(|| name.clone());

        // looks so bad but basically it joins both `ports` and `expose` into a single list
        // then parses the port if its port:port or port format
        let gateways = {
            let ports = HashSet::<_>::from_iter(
                service
                    .expose
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .chain(service.ports.clone().unwrap_or_default().into_iter()),
            );

            log::debug!("Found ports: {:?}", ports);

            let mut gateways = vec![];

            for port in ports {
                log::info!("Found port `{port}` in the compose file for `{name}`");

                let config = GatewayConfig {
                    target_port: Some(port.0),
                    internal_domain: Some(format!("{dep_name}.hop")),
                    ..Default::default()
                };

                let gateway_config =
                    update_gateway_config(&Default::default(), false, false, &config)?;

                gateways.push(gateway_config);
            }

            gateways
        };

        if deployment_config.0.image.name.is_empty() {
            log::info!("The image for `{name}` will be built by the Hop CLI and pushed to the Hop registry");

            deployment_config.0.image.name =
                format!("{}/{}/{}", HOP_REGISTRY_URL, project.namespace, dep_name);
        }

        deployments_with_gateway.push((
            deployment_config.0,
            deployment_config.1,
            service.build.clone(),
            gateways,
        ));

        // add a new line
        println!();
    }

    let has_unbuilt = deployments_with_gateway.iter().any(|(_, _, build, _)| {
        if build.is_some() {
            return true;
        }

        false
    });

    let build_localy = if has_unbuilt {
        let answer = dialoguer::Confirm::new()
            .with_prompt("Some of the services in the compose file require building. Would you like to build them locally?")
            .default(true)
            .interact()?;

        if answer {
            log::info!("Building images locally");
        } else {
            log::info!("Images will be built by the Hop Builder");
        }

        println!();

        answer
    } else {
        false
    };

    for (deployment, containers, builder, gateways) in deployments_with_gateway {
        let dep = create_deployment(&state.http, &project.id, &deployment).await?;
        log::info!("Created deployment `{}`", dep.name);

        if let Some(build) = builder {
            let path = match build {
                ServiceBuildUnion::Map { context, .. } => context,
                ServiceBuildUnion::String(context) => context,
            }
            .parse::<PathBuf>()?;

            log::info!("Building image for `{}`", dep.name);

            let path = if path != PathBuf::from(".") {
                parent_dir.join(path)
            } else {
                parent_dir.to_path_buf()
            };

            HopFile::new(path.join("hop.yml"), &project.id, &dep.id)
                .save()
                .await?;

            log::info!("Created hop.yml for `{}`", dep.name);

            if build_localy {
                local::build(&state, &dep.config.image.name, path).await?;
            } else {
                builder::build(&state, &project.id, &dep.id, path).await?;
            }
        }

        if let Some(count) = containers.containers {
            if dep.can_scale() && count > 0 {
                scale(&state.http, &dep.id, count).await?;

                log::info!("Created {count} containers");
            }
        }

        for gateway in gateways {
            create_gateway(&state.http, &dep.id, &gateway).await?;
            log::info!("Created gateway for `{}`", dep.name);
        }
    }

    log::info!("Finished creating deployments from {}", file.display());
    log::info!("You can view the deployments by running `hop ignite ls --project {}`", project.namespace);

    Ok(())
}
