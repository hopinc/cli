use anyhow::{bail, ensure, Result};
use clap::Parser;
use serde_json::Value;

use crate::commands::secrets::types::Secrets;
use crate::commands::secrets::utils::validate_name;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete a secret")]
pub struct Options {
    #[clap(help = "Name of the secret")]
    name: Option<String>,
    #[clap(short, long, help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    if let Some(ref name) = options.name {
        validate_name(name).unwrap();
    }

    let project_id = state.ctx.current_project_error()?.id;

    let secret_name = match options.name {
        Some(name) => name,
        None => {
            let secrets = state
                .http
                .request::<Secrets>("GET", &format!("/projects/{project_id}/secrets"), None)
                .await?
                .unwrap()
                .secrets;

            ensure!(!secrets.is_empty(), "No secrets found");

            let secrets_fmt = secrets
                .iter()
                .map(|s| format!(" {} ({})", s.name, s.id))
                .collect::<Vec<_>>();

            let idx = dialoguer::Select::new()
                .with_prompt("Select a secret")
                .items(&secrets_fmt)
                .default(0)
                .interact()?;

            secrets[idx].name.clone()
        }
    };

    if !options.force
        && !dialoguer::Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete secret `{secret_name}`?"
            ))
            .interact_opt()?
            .unwrap_or(false)
    {
        bail!("Aborted");
    }

    state
        .http
        .request::<Value>(
            "DELETE",
            &format!("/projects/{project_id}/secrets/{secret_name}"),
            None,
        )
        .await?;

    log::info!("Secret `{}` deleted", secret_name);

    Ok(())
}
