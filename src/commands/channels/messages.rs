use anyhow::{anyhow, ensure, Result};
use clap::Parser;

use super::{
    types::EventOptions,
    utils::{format_channels, get_all_channels, message_channel},
};
use crate::state::State;

#[derive(Debug, Parser, Default, PartialEq)]
#[clap(about = "Send a message to a Channel")]
pub struct Options {
    #[clap(
        short = 'c',
        long = "channel",
        help = "The ID of the Channel to send the message to"
    )]
    pub channel: Option<String>,

    #[clap(flatten)]
    pub event: EventOptions,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project_id = state.ctx.current_project_error().id;

    let channel_id = if let Some(channel) = options.channel {
        channel
    } else {
        let channels = get_all_channels(&state.http, &project_id).await?;
        ensure!(
            !channels.is_empty(),
            "No Channels found in the current Project"
        );
        let channels_fmt = format_channels(&channels, false);

        let idx = dialoguer::Select::new()
            .with_prompt("Select a Channel")
            .items(&channels_fmt)
            .default(0)
            .interact_opt()?
            .ok_or_else(|| anyhow::anyhow!("No Channel selected"))?;

        channels.get(idx).unwrap().id.clone()
    };

    let (event_name, event_data) = if options.event != EventOptions::default() {
        (
            options.event.name.ok_or_else(|| {
                anyhow!("The argument '--event <EVENT>' requires a value but none was supplied")
            })?,
            options
                .event
                .data
                .map(|d| serde_json::from_str(&d).unwrap()),
        )
    } else {
        let event_name = dialoguer::Input::<String>::new()
            .with_prompt("Enter the event to send to the Channel")
            .interact_text()?;

        let event_data = if dialoguer::Confirm::new()
            .with_prompt("Do you want to specify event data?")
            .default(false)
            .interact()?
        {
            let editor_cmd = std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| "vim".to_string());

            loop {
                match serde_json::to_value(
                    dialoguer::Editor::new()
                        .executable(&editor_cmd)
                        .require_save(true)
                        .edit("")?,
                ) {
                    Ok(event_data) => break Some(event_data),
                    Err(e) => {
                        log::error!("Invalid JSON: {}", e);
                    }
                }
            }
        } else {
            None
        };

        (event_name, event_data)
    };

    message_channel(
        &state.http,
        &project_id,
        &channel_id,
        &event_name,
        event_data,
    )
    .await?;

    log::info!("Message sent to Channel `{}`", channel_id);

    Ok(())
}
