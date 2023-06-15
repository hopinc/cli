mod utils;

use anyhow::{bail, Result};
use clap::Parser;

use crate::commands::projects::create::utils::get_payment_method_from_user;
use crate::commands::projects::utils::{create_project, format_project, validate_namespace};
use crate::state::State;
use crate::store::Store;

// TODO: replace when ../new path is implemented
const WEB_PAYMENTS_URL: &str = "https://console.hop.io/settings/cards";

#[derive(Debug, Parser)]
#[clap(about = "Create a new project")]
#[group(skip)]
pub struct Options {
    #[clap(help = "Namespace of the project")]
    namespace: Option<String>,
    #[clap(help = "Name of the project")]
    name: Option<String>,
    #[clap(short, long, help = "Set as default project")]
    default: bool,
}

pub async fn handle(options: Options, mut state: State) -> Result<()> {
    let namespace = if let Some(namespace) = options.namespace {
        namespace
    } else {
        dialoguer::Input::new()
            .with_prompt("Namespace of the project")
            .validate_with(|input: &String| -> Result<()> { validate_namespace(input) })
            .interact_text()?
    };

    let name = if let Some(name) = options.name {
        name
    } else {
        dialoguer::Input::new()
            .with_prompt("Name of the project")
            .validate_with(|input: &String| -> Result<()> {
                if input.len() > 32 {
                    bail!("Project name must be less than 32 characters")
                }

                Ok(())
            })
            .interact_text()?
    };

    let payment_method_id = get_payment_method_from_user(&state.http).await?;

    let project = create_project(&state.http, &name, &namespace, &payment_method_id).await?;

    if options.default {
        state.ctx.default_project = Some(project.id.clone());
        state.ctx.save().await?;
    }

    log::info!("Created project {}", format_project(&project));

    Ok(())
}
