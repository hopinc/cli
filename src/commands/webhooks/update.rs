use anyhow::{Context, Result};
use clap::Parser;
use hop::webhooks::types::{PossibleEvents, EVENT_NAMES};

use super::utils::string_to_event;
use crate::commands::webhooks::utils::{format_webhooks, get_formatted_events};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Update a webhook")]
#[group(skip)]
pub struct Options {
    #[clap(short, long, help = "The id of the webhook")]
    pub id: Option<String>,
    #[clap(short, long, help = "The url to send the webhook to")]
    pub url: Option<String>,
    #[clap(short, long, help = "The events to send the webhook on", value_parser = string_to_event )]
    pub events: Vec<PossibleEvents>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project = state.ctx.current_project_error()?;

    let all = state.hop.webhooks.get_all(&project.id).await?;

    let old = if let Some(id) = options.id {
        all.into_iter()
            .find(|webhook| webhook.id == id)
            .context("Webhook not found")?
    } else {
        let formatted_webhooks = format_webhooks(&all, false);

        let idx = dialoguer::Select::new()
            .with_prompt("Select a webhook")
            .items(&formatted_webhooks)
            .default(0)
            .interact()?;

        all[idx].clone()
    };

    let url = if let Some(url) = options.url {
        url
    } else {
        dialoguer::Input::new()
            .with_prompt("Webhook URL")
            .default(old.webhook_url)
            .interact_text()?
    };

    let events = if !options.events.is_empty() {
        options.events
    } else {
        let dialoguer_events = loop {
            let idxs = dialoguer::MultiSelect::new()
                .with_prompt("Select events")
                .items(&get_formatted_events()?)
                .defaults(&EVENT_NAMES.map(|(event, _)| old.events.contains(&event)))
                .interact()?;

            if !idxs.is_empty() {
                break idxs;
            }
        };

        EVENT_NAMES
            .into_iter()
            .enumerate()
            .filter(|(idx, _)| dialoguer_events.binary_search(idx).is_ok())
            .map(|(_, (event, _))| event)
            .collect()
    };

    let webhook = state
        .hop
        .webhooks
        .patch(&project.id, &old.id, &url, &events)
        .await?;

    log::info!("Webhook successfully created. ID: {}", webhook.id);

    Ok(())
}
