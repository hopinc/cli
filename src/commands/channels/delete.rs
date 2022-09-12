use anyhow::{anyhow, bail, ensure, Result};
use clap::Parser;

use super::utils::delete_channel;
use crate::commands::channels::utils::{format_channels, get_all_channels};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete Channels")]
pub struct Options {
    #[clap(name = "channels", help = "IDs of the Channels", min_values = 0)]
    channels: Vec<String>,

    #[clap(short = 'f', long = "force", help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project_id = state.ctx.current_project_error().id;

    let channels = if !options.channels.is_empty() {
        options.channels
    } else {
        let channels = get_all_channels(&state.http, &project_id).await?;
        ensure!(!channels.is_empty(), "No Channels found");
        let channels_fmt = format_channels(&channels, false);

        let idxs = dialoguer::MultiSelect::new()
            .with_prompt("Select a Channel")
            .items(&channels_fmt)
            .interact_opt()?
            .ok_or_else(|| anyhow!("No channel selected"))?;

        channels
            .iter()
            .enumerate()
            .filter(|(i, _)| idxs.contains(i))
            .map(|(_, c)| c.id.clone())
            .collect()
    };

    if !options.force
        && !dialoguer::Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete {} Channels?",
                channels.len()
            ))
            .default(false)
            .interact_opt()?
            .unwrap_or(false)
    {
        bail!("Aborted");
    }

    let mut delete_count = 0;

    for channel in &channels {
        log::debug!("{channel:?}");

        log::info!("Deleting Channel `{}`", channel);

        if let Err(err) = delete_channel(&state.http, &project_id, channel).await {
            log::error!("Failed to delete Channel `{}`: {}", channel, err);
        } else {
            delete_count += 1;
        }
    }

    log::info!("Deleted {delete_count}/{} Channels", channels.len());

    Ok(())
}
