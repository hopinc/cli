use anyhow::{anyhow, Result};
use clap::Parser;
use regex::Regex;

use super::create::DeploymentConfig;
use crate::commands::containers::utils::create_containers;
use crate::commands::ignite::create::Options as CreateOptions;
use crate::commands::ignite::types::{Config, Deployment, Image, MapTo, PremadeInput, Volume};
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

    let premades = get_premade(&state.http).await?;

    let premade = if let Some(ref template) = options.template {
        premades
            .iter()
            .find(|p| p.name.to_lowercase() == template.to_lowercase())
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

    if let Some(form) = &premade.form {
        log::info!("This template requires some additional information");

        for field in &form.fields {
            let value = match &field.input {
                PremadeInput::String {
                    default,
                    autogen,
                    max_length,
                    validator,
                } => {
                    let mut input = dialoguer::Input::<String>::new();

                    if let Some(default) = default {
                        input.default(default.clone());
                    }

                    input.validate_with(|input: &String| -> Result<(), String> {
                        if let Some(max_length) = *max_length {
                            if input.len() > max_length {
                                return Err(format!(
                                    "Input must be less than {max_length} characters",
                                ));
                            }
                        }

                        if let Some(validator) = validator.clone() {
                            if Regex::new(&validator)
                                .map_err(|e| e.to_string())?
                                .is_match(input)
                            {
                                return Err(format!("Input must match regex `{validator}`",));
                            }
                        }

                        Ok(())
                    });

                    if let Some(description) = &field.description {
                        input.with_prompt(format!("{} ({})", field.title, description));
                    } else {
                        input.with_prompt(&field.title);
                    }

                    input.interact()?
                }
            };

            for place in &field.map_to {
                match place {
                    MapTo::Env { key } => {
                        deployment_config.env.insert(key.clone(), value.clone());
                    }
                }
            }
        }
    }

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
