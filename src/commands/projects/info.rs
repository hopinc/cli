use anyhow::Result;
use clap::Parser;

use crate::commands::projects::utils::format_project;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Get information about a project")]
pub struct Options {}

pub fn handle(_options: &Options, state: State) -> Result<()> {
    let project = state.ctx.current_project_error()?;

    log::info!("Project: {}", format_project(&project));

    Ok(())
}
