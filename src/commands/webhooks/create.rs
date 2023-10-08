use anyhow::{Context, Result};
use clap::Parser;
use hop::webhooks::types::{PossibleEvents, EVENT_CATEGORIES, EVENT_NAMES};

use super::utils::string_to_event;
use crate::state::State;
use crate::utils::urlify;

#[derive(Debug, Parser)]
#[clap(about = "Create a new webhook")]
#[group(skip)]
pub struct Options {
    #[clap(short, long, help = "The url to send the webhook to")]
    pub url: Option<String>,
    #[clap(short, long, help = "The events to send the webhook on", value_parser = string_to_event )]
    pub events: Vec<PossibleEvents>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project = state.ctx.current_project_error()?;

    let url = if let Some(url) = options.url {
        url
    } else {
        dialoguer::Input::new()
            .with_prompt("Webhook URL")
            .interact_text()?
    };

    let events = if !options.events.is_empty() {
        options.events
    } else {
        let mut events = vec![];
        let mut start_idx = 0usize;

        for (name, end_idx) in EVENT_CATEGORIES {
            let end_idx = end_idx as usize + start_idx;

            for (_, event) in &EVENT_NAMES[start_idx..end_idx] {
                events.push(format!("{name}: {event}"))
            }

            start_idx = end_idx;
        }

        let dialoguer_events = loop {
            let test = dialoguer::MultiSelect::new()
                .with_prompt("Select events")
                .items(&events)
                .interact()?;

            if !test.is_empty() {
                break test;
            }
        };

        EVENT_NAMES
            .into_iter()
            .enumerate()
            .filter(|(idx, _)| dialoguer_events.contains(idx))
            .map(|(_, (event, _))| event)
            .collect()
    };

    let webhook = state
        .hop
        .webhooks
        .create(&project.id, &url, &events)
        .await?;

    log::info!("Webhook successfully created. ID: {}\n", webhook.id);
    log::info!("This is your webhook's secret, this is how you will authenticate traffic coming to your endpoint");
    log::info!("Webhook Header: {}", urlify("X-Hop-Hooks-Signature"));
    log::info!(
        "Webhook Secret: {}",
        urlify(
            &webhook
                .secret
                .context("Webhook secret was expected to be present")?
        )
    );

    Ok(())
}
