use anyhow::{ensure, Result};
use clap::Parser;

use crate::commands::secrets::types::Secrets;
use crate::commands::secrets::util::validate_name;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete a secret")]
pub struct Options {
    #[clap(name = "name", help = "Name of the secret")]
    pub name: Option<String>,
    #[clap(long = "no-confirm", help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    if let Some(ref name) = options.name {
        validate_name(name).unwrap();
    }

    let project_id = state.ctx.current_project_error().id;

    let secret_name = match options.name {
        Some(name) => name,
        None => {
            let secrests = state
                .http
                .request::<Secrets>("GET", &format!("/projects/{}/secrets", project_id), None)
                .await?
                .unwrap()
                .secrets;

            assert!(!secrests.is_empty(), "No secrets found");

            let secrets_fmt = secrests
                .iter()
                .map(|s| format!(" {} ({})", s.name, s.id))
                .collect::<Vec<_>>();

            let idx = dialoguer::Select::new()
                .with_prompt("Select a secret to delete")
                .items(&secrets_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select secret")
                .expect("No secret selected");

            secrests[idx].name.clone()
        }
    };

    if !options.force {
        let confirm = dialoguer::Confirm::new()
            .with_prompt(&format!(
                "Are you sure you want to delete secret `{}`?",
                secret_name
            ))
            .interact_opt()?;

        ensure!(
            (confirm.is_some() || confirm.unwrap()),
            "Aborted deletion of `{}`",
            secret_name
        );
    }

    state
        .http
        .request::<()>(
            "DELETE",
            &format!("/projects/{}/secrets/{}", project_id, secret_name),
            None,
        )
        .await?;

    log::info!("Secret `{}` deleted", secret_name);

    Ok(())
}
