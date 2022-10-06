use anyhow::{ensure, Result};
use clap::Parser;

use super::utils::{format_channels, get_all_channels, subscribe_to_channel};
use crate::commands::channels::tokens::utils::{format_tokens, get_all_tokens};
use crate::state::State;

#[derive(Debug, Parser, Default, PartialEq, Eq)]
#[clap(about = "Subscribe a Leap Token to a Channel")]
pub struct Options {
    #[clap(
        short = 'c',
        long = "channel",
        help = "The ID of the Channel to subscribe to"
    )]
    pub channel: Option<String>,
    #[clap(
        short = 't',
        long = "token",
        help = "The ID of the Leap Token to subscribe to the Channel"
    )]
    pub token: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project_id = state.ctx.current_project_error().id;

    let channel_id = if let Some(channel_id) = options.channel {
        channel_id
    } else {
        let channels = get_all_channels(&state.http, &project_id).await?;
        ensure!(
            !channels.is_empty(),
            "No Channels found in Project '{}'",
            project_id
        );
        let channels_fmt = format_channels(&channels, false);

        let idx = dialoguer::Select::new()
            .with_prompt("Select a Channel")
            .items(&channels_fmt)
            .default(0)
            .interact()?;

        channels[idx].id.clone()
    };

    let token_id = if let Some(token_id) = options.token {
        token_id
    } else {
        let tokens = get_all_tokens(&state.http, &project_id).await?;
        ensure!(
            !tokens.is_empty(),
            "No Leap Tokens found in Project '{}'",
            project_id
        );
        let tokens_fmt = format_tokens(&tokens, false);

        let idx = dialoguer::Select::new()
            .with_prompt("Select a Leap Token")
            .items(&tokens_fmt)
            .default(0)
            .interact()?;

        tokens[idx].id.clone()
    };

    subscribe_to_channel(&state.http, &project_id, &channel_id, &token_id).await?;

    log::info!(
        "Subscribed Token '{}' to Channel '{}'",
        token_id,
        channel_id
    );

    Ok(())
}
