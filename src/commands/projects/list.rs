use clap::Parser;

use super::util::format_projects;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List all projects")]
pub struct Options {
    #[clap(
        short = 'q',
        long = "quiet",
        help = "Only print the IDs of the projects"
    )]
    pub quiet: bool,
}

pub fn handle(options: &Options, state: State) {
    let projects = state.ctx.current.unwrap().projects;

    if options.quiet {
        let ids = projects
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{}", ids);
    } else {
        let projects_fmt = format_projects(&projects, true);

        println!("{}", projects_fmt.join("\n"));
    }
}
