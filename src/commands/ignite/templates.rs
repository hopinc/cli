use anyhow::{anyhow, Result};
use clap::Parser;
use rand::Rng;
use regex::Regex;

use super::create::DeploymentConfig;
use crate::commands::containers::utils::create_containers;
use crate::commands::ignite::create::Options as CreateOptions;
use crate::commands::ignite::types::{
    Autogen, Config, Deployment, Image, MapTo, PremadeInput, Volume,
};
use crate::commands::ignite::utils::{
    create_deployment, format_premade, get_premade, update_deployment_config, WEB_IGNITE_URL,
};
use crate::commands::projects::utils::format_project;
use crate::state::State;
use crate::utils::urlify;

#[derive(Debug, Parser, Default, PartialEq, Clone)]
#[clap(about = "Create a new deployment")]
#[group(skip)]
pub struct Options {
    #[clap(flatten)]
    pub config: DeploymentConfig,

    #[clap(help = "Name of the template to use")]
    pub template: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project = state.ctx.current_project_error()?;

    log::info!("Deploying to project {}", format_project(&project));

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

    let mut deployment = Deployment {
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
    };

    if let Some(form) = &premade.form {
        log::info!("This template requires some additional information");

        for field in &form.fields {
            let value = match &field.input {
                PremadeInput::String {
                    default,
                    autogen,
                    max_length,
                    validator,
                    required,
                } => {
                    let mut input = dialoguer::Input::<String>::new();

                    if let Some(default) = default {
                        input.default(default.clone());
                    } else if let Some(autogen) = autogen {
                        input.default(match autogen {
                            Autogen::ProjectNamespace => project.namespace.clone(),

                            Autogen::SecureToken => {
                                // generate random bits securely
                                let mut rng = rand::thread_rng();

                                // generate a random string of 24 characters
                                std::iter::repeat(())
                                    .map(|()| rng.sample(rand::distributions::Alphanumeric))
                                    .take(24)
                                    .map(|b| b as char)
                                    .collect()
                            }
                        });
                    }

                    input.validate_with(|input: &String| -> Result<(), String> {
                        if input.len() > *max_length {
                            return Err(
                                format!("Input must be less than {max_length} characters",),
                            );
                        }

                        let validator = {
                            let valid = validator.split('/').collect::<Vec<_>>();

                            if valid.len() == 3 {
                                valid[1]
                            } else {
                                return Err(format!("Invalid validator `{validator}`",));
                            }
                        };

                        if !Regex::new(validator)
                            .map_err(|e| e.to_string())?
                            .is_match(input)
                        {
                            return Err(format!("Input must match regex `{validator}`",));
                        }

                        Ok(())
                    });

                    if let Some(description) = &field.description {
                        input.with_prompt(format!("{} ({})", field.title, description));
                    } else {
                        input.with_prompt(&field.title);
                    }

                    input.allow_empty(!required);

                    let value = input.interact()?;

                    if *required {
                        value
                    } else {
                        continue;
                    }
                }

                PremadeInput::Range {
                    default,
                    min,
                    max,
                    increment,
                    unit,
                } => {
                    let items = std::iter::repeat(())
                        .enumerate()
                        .map(|(i, _)| format!("{}{}", min + (i as u64 * increment), unit))
                        .take(((max - min) / increment) as usize)
                        .collect::<Vec<_>>();

                    let mut input = dialoguer::Select::new();

                    input.default(
                        items
                            .iter()
                            .position(|i| i == &format!("{default}{unit}"))
                            .unwrap_or(0),
                    );

                    input.with_prompt(&field.title);

                    input.items(&items);

                    items[input.interact()?].clone()
                }
            };

            for place in &field.map_to {
                match place {
                    MapTo::Env { key } => {
                        deployment.config.env.insert(key.clone(), value.clone());
                    }
                    MapTo::VolumeSize => {
                        deployment.config.volume = deployment.config.volume.take().map(|mut v| {
                            v.size = value.clone();
                            v
                        });
                    }
                }
            }
        }
    }

    let (mut deployment_config, container_options) = update_deployment_config(
        &state.http,
        CreateOptions {
            config: options.config.clone(),
            // temporary value that gets replaced after we get the name
            image: Some("".to_string()),
        },
        options == Options::default(),
        &deployment,
        &Some(premade.name.clone()),
        false,
        &project,
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
