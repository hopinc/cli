use anyhow::Result;
use clap::Parser;

use crate::{state::State, util::validate_json};

use super::utils::create_token;

#[derive(Debug, Parser, Default, PartialEq, Eq)]
#[clap(about = "Create a new Leap Token")]
pub struct Options {
    #[clap(
        short = 'e',
        long = "expiration",
        help = "Expiration date of the token, can be a date (DD/MM/YYYY, DD-MM-YYYY) or a duration (1D, 1M, 1Y)",
        validator = validate_json
    )]
    pub expires_at: Option<String>,

    #[clap(
        short = 's',
        long = "state",
        help = "Initial state of the token, can be any JSON value"
    )]
    pub state: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project_id = state.ctx.current_project_error().id;

    let (token_state, expires_at) = if options != Options::default() {
        (options.state, options.expires_at)
    } else {
        let token_state = dialoguer::Input::<String>::new()
            .with_prompt("State")
            .default("null".to_string())
            .validate_with(|s: &String| validate_json(s))
            .interact_text()?;

        let expires_at = dialoguer::Input::<String>::new()
            .with_prompt("Expiration date")
            .default("0".to_string())
            .interact_text()?;

        (
            if token_state.to_lowercase() == "null" {
                None
            } else {
                Some(token_state)
            },
            if expires_at.to_lowercase() == "0" {
                None
            } else {
                Some(expires_at)
            },
        )
    };

    let token = create_token(
        &state.http,
        &project_id,
        expires_at.as_deref(),
        token_state.map(|s| s.parse().unwrap()),
    )
    .await?;

    log::info!("Created token: `{}`", token.id);

    Ok(())
}
