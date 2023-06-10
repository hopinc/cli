use anyhow::{Context, Result};
use clap::Parser;

use super::utils::format_projects;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List all projects")]
#[group(skip)]
pub struct Options {
    #[clap(short, long, help = "Only print the IDs of the projects")]
    pub quiet: bool,
}

pub fn handle(options: Options, state: State) -> Result<()> {
    let projects = state.ctx.current.context("You are not logged in")?.projects;

    if options.quiet {
        let ids = projects
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{ids}");
    } else {
        let projects_fmt = format_projects(&projects, true);

        println!("{}", projects_fmt.join("\n"));
    }

    Ok(())
}
