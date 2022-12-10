use anyhow::{bail, Result};
use clap::Parser;
use console::Term;

use crate::commands::auth::payment::utils::{format_payment_methods, get_all_payment_methods};
use crate::commands::projects::utils::{create_project, format_project, validate_namespace};
use crate::state::State;
use crate::utils::urlify;

// TODO: replace when ../new path is implemented
const WEB_PAYMENTS_URL: &str = "https://console.hop.io/settings/cards";

#[derive(Debug, Parser)]
#[clap(about = "Create a new project")]
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
                } else {
                    Ok(())
                }
            })
            .interact_text()?
    };

    let payment_method_id;

    loop {
        let payment_methods = get_all_payment_methods(&state.http).await?;
        let mut payment_methods_fmt = format_payment_methods(&payment_methods, false)?;
        payment_methods_fmt.push("New payment method".to_string());

        let payment_method_idx = dialoguer::Select::new()
            .with_prompt("Select a payment method")
            .items(&payment_methods_fmt)
            .default(0)
            .interact()?;

        if payment_method_idx == payment_methods_fmt.len() - 1 {
            let _ = Term::stderr().clear_last_lines(1);
            log::info!(
                "To add a new payment method, please visit {}. You can then come back here and select it.",
                urlify(WEB_PAYMENTS_URL)
            );

            log::info!("Press enter to continue");

            let _ = std::io::stdin().read_line(&mut String::new())?;

            // clear 3 because 2 logs and 1 new line
            let _ = Term::stdout().clear_last_lines(3);
        } else {
            payment_method_id = payment_methods[payment_method_idx].id.clone();

            break;
        }
    }

    let project = create_project(&state.http, &name, &namespace, &payment_method_id).await?;

    if options.default {
        state.ctx.default_project = Some(project.id.clone());
        state.ctx.save().await?;
    }

    log::info!("Created project {}", format_project(&project));

    Ok(())
}
