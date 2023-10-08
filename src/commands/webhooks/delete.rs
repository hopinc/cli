use anyhow::{Context, Result};
use clap::Parser;

use crate::commands::webhooks::utils::format_webhooks;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete a webhook")]
#[group(skip)]
pub struct Options {
    #[clap(short, long, help = "The id of the webhook")]
    pub id: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project = state.ctx.current_project_error()?;

    let all = state.hop.webhooks.get_all(&project.id).await?;

    let webhook = if let Some(id) = options.id {
        all.into_iter()
            .find(|webhook| webhook.id == id)
            .context("Webhook not found")?
    } else {
        let formatted_webhooks = format_webhooks(&all, false);

        let idx = dialoguer::Select::new()
            .with_prompt("Select a webhook to update")
            .items(&formatted_webhooks)
            .default(0)
            .interact()?;

        all[idx].clone()
    };

    state.hop.webhooks.delete(&project.id, &webhook.id).await?;

    Ok(())
}
