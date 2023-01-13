use anyhow::{anyhow, Result};
use clap::Parser;

use super::create::DeploymentConfig;
use crate::commands::containers::utils::create_containers;
use crate::commands::ignite::create::Options as CreateOptions;
use crate::commands::ignite::types::{Config, Deployment, Image, Volume};
use crate::commands::ignite::utils::{
    create_deployment, format_premade, get_premade, update_deployment_config, WEB_IGNITE_URL,
};
use crate::state::State;
use crate::utils::urlify;

#[derive(Debug, Parser, Default, PartialEq, Clone)]
#[clap(about = "Create a new deployment")]
pub struct Options {
    #[clap(flatten)]
    pub config: DeploymentConfig,

    #[clap(help = "Name of the template to use")]
    pub template: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project = state.ctx.current_project_error();

    log::info!(
        "Deploying to project {} /{} ({})",
        project.name,
        project.namespace,
        project.id
    );

    let premades = get_premade(&state.http).await?;

    let premade = if let Some(ref template) = options.template {
        premades
            .iter()
            .find(|p| &p.name == template)
            .ok_or_else(|| anyhow!("Could not find template `{}`", template))?
    } else {
        let premade_fmt = format_premade(&premades, false)?;

        let selection = dialoguer::Select::new()
            .with_prompt("Select a template")
            .items(&premade_fmt)
            .default(0)
            .interact()?;

        &premades[selection]
    };

    // log::info!("Using template `{}`", premade.name);

    let (mut deployment_config, container_options) = update_deployment_config(
        &state.http,
        CreateOptions {
            config: options.config.clone(),
            // temporary value that gets replaced after we get the name
            image: Some("".to_string()),
        },
        options == Options::default(),
        &Deployment {
            config: Config {
                entrypoint: premade.entrypoint.clone(),
                env: premade.environment.clone().unwrap_or_default(),
                volume: Some(Volume {
                    fs: premade.filesystem.clone().unwrap_or_default(),
                    mount_path: premade.mountpath.clone(),
                    size: "".to_string(),
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        &Some(premade.name.clone()),
        false,
    )
    .await?;

    // override the image with the premade image
    deployment_config.image = Some(Image {
        name: premade.image.clone(),
    });

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

    if let Some(ref final_note) = premade.final_note {
        log::info!("{}", final_note);
    }

    Ok(())
}
