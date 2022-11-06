use anyhow::{anyhow, bail, ensure, Result};
use clap::Parser;

use super::utils::{delete_token, format_tokens, get_all_tokens};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete Leap Tokens")]
pub struct Options {
    #[clap(help = "IDs of the Leap Tokens")]
    tokens: Vec<String>,

    #[clap(short, long, help = "Skip confirmation")]
    force: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project_id = state.ctx.current_project_error().id;

    let tokens = if !options.tokens.is_empty() {
        options.tokens
    } else {
        let tokens = get_all_tokens(&state.http, &project_id).await?;
        ensure!(!tokens.is_empty(), "No Leap Tokens found");
        let tokens_fmt = format_tokens(&tokens, false);

        let idxs = dialoguer::MultiSelect::new()
            .with_prompt("Select a Leap Token")
            .items(&tokens_fmt)
            .interact_opt()?
            .ok_or_else(|| anyhow!("No token selected"))?;

        tokens
            .iter()
            .enumerate()
            .filter(|(i, _)| idxs.contains(i))
            .map(|(_, c)| c.id.clone())
            .collect()
    };

    if !options.force
        && !dialoguer::Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete {} Leap Tokens?",
                tokens.len()
            ))
            .default(false)
            .interact_opt()?
            .unwrap_or(false)
    {
        bail!("Aborted");
    }

    let mut delete_count = 0;

    for token in &tokens {
        log::debug!("{token:?}");

        log::info!("Deleting Leap Token `{}`", token);
        delete_token(&state.http, &project_id, token).await?;
        delete_count += 1;
    }

    log::info!("Deleted {delete_count}/{} Leap Tokens", tokens.len());

    Ok(())
}
