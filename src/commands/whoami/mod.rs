use anyhow::{anyhow, Result};
use clap::Parser;

use crate::config::EXEC_NAME;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Get information about the current user")]
pub struct Options {}

pub fn handle(_options: &Options, state: State) -> Result<()> {
    let authorized = state
        .ctx
        .current
        .clone()
        .ok_or_else(|| anyhow!("You are not logged in"))?;

    log::info!(
        "You are logged in as `{}` ({})",
        authorized.name,
        authorized.email.unwrap_or(authorized.id)
    );

    let project = state.ctx.current_project();

    match project {
        Some(project) => {
            log::info!(
                "Project: `{}` ({}) {}",
                project.name,
                project.id,
                project.p_type
            );
        }
        None => {
            log::warn!(
                "No project is currently selected. Please run `{} projects switch` first.",
                EXEC_NAME
            );
        }
    }

    Ok(())
}
