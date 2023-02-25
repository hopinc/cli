use anyhow::{anyhow, Result};
use clap::Parser;

use crate::commands::projects::info;
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
        authorized.email
    );

    info::handle(&info::Options {}, state)
}
