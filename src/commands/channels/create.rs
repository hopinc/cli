use anyhow::{anyhow, Result};
use clap::Parser;
use serde_json::Value;

use super::types::ChannelType;
use crate::commands::channels::utils::create_channel;
use crate::state::State;
use crate::utils::validate_json_non_null;

#[derive(Debug, Parser, Default, PartialEq, Eq)]
#[clap(about = "Create a new Channel")]
pub struct Options {
    #[clap(short = 'i', long = "id", help = "Custom ID for the channel")]
    custom_id: Option<String>,

    #[clap(short = 't', long = "type", help = "Type of the channel")]
    channel_type: Option<ChannelType>,

    #[clap(short, long, help = "Initial state of the channel", value_parser = validate_json_non_null )]
    state: Option<Value>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project_id = state.ctx.current_project_error()?.id;

    let (type_, id, init_state) = if Options::default() == options {
        let type_ = dialoguer::Select::new()
            .with_prompt("Select a channel type")
            .items(&ChannelType::variants())
            .default(0)
            .interact()?;

        let type_ = ChannelType::variants()[type_].clone();

        let id = if dialoguer::Confirm::new()
            .with_prompt("Do you want to specify a custom Channel ID?")
            .default(false)
            .interact()?
        {
            Some(
                dialoguer::Input::<String>::new()
                    .with_prompt("Enter a custom ID")
                    .interact()?,
            )
        } else {
            None
        };

        let state = dialoguer::Input::new()
            .with_prompt("Enter the initial state of the channel")
            .default("{}".to_string())
            .validate_with(|s: &String| -> Result<(), String> {
                validate_json_non_null(s)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            })
            .interact()?;

        let state = serde_json::from_str(&state)?;

        (type_, id, state)
    } else {
        (
            options.channel_type.clone().ok_or_else(|| {
                anyhow!(
                    "The argument '--type <CHANNELTYPE>' requires a value but none was supplied"
                )
            })?,
            options.custom_id.clone(),
            options
                .state
                .clone()
                .unwrap_or_else(|| Value::Object(serde_json::Map::new())),
        )
    };

    let channel =
        create_channel(&state.http, &project_id, &type_, &init_state, id.as_deref()).await?;

    log::info!("Created Channel `{}`", channel.id);

    Ok(())
}
