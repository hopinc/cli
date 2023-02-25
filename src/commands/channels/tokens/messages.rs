use anyhow::{anyhow, ensure, Result};
use clap::Parser;

use super::utils::{format_tokens, get_all_tokens, message_token};
use crate::commands::channels::types::EventOptions;
use crate::commands::channels::utils::get_json_input;
use crate::state::State;

#[derive(Debug, Parser, Default, PartialEq, Eq)]
#[clap(about = "Send a message to a Leap Token")]
pub struct Options {
    #[clap(short, long, help = "The ID of the Token to send the message to")]
    token: Option<String>,

    #[clap(flatten)]
    event: EventOptions,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project_id = state.ctx.current_project_error()?.id;

    let token_id = if let Some(token) = options.token {
        token
    } else {
        let tokens = get_all_tokens(&state.http, &project_id).await?;
        ensure!(
            !tokens.is_empty(),
            "No Leap Tokens found in the current Project"
        );
        let channels_fmt = format_tokens(&tokens, false);

        let idx = dialoguer::Select::new()
            .with_prompt("Select a Leap Token")
            .items(&channels_fmt)
            .default(0)
            .interact()?;

        tokens.get(idx).unwrap().id.clone()
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
            .with_prompt("Enter the event name to send")
            .interact_text()?;

        let event_data = if dialoguer::Confirm::new()
            .with_prompt("Do you want to specify event data?")
            .default(false)
            .interact()?
        {
            Some(get_json_input()?)
        } else {
            None
        };

        log::debug!("Event: {} Data: {:?}", event_name, event_data);

        (event_name, event_data)
    };

    message_token(&state.http, &project_id, &token_id, &event_name, event_data).await?;

    log::info!("Message sent to Leap Token `{}`", token_id);

    Ok(())
}
