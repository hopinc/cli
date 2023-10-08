use anyhow::{Context, Result};
use clap::Parser;

use crate::commands::webhooks::utils::format_webhooks;
use crate::state::State;
use crate::utils::urlify;

#[derive(Debug, Parser)]
#[clap(about = "Regenerate a webhook secret")]
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

    let token = state
        .hop
        .webhooks
        .regenerate_secret(&project.id, &webhook.id)
        .await?;

    log::info!("This is your webhook's secret, this is how you will authenticate traffic coming to your endpoint");
    log::info!("Webhook Header: {}", urlify("X-Hop-Hooks-Signature"));
    log::info!("Webhook Secret: {}", urlify(&token));

    Ok(())
}
