use clap::Parser;

use super::util::format_projects;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List all projects")]
pub struct ListOptions {
    #[clap(
        short = 'q',
        long = "quiet",
        help = "Only print the IDs of the projects"
    )]
    pub quiet: bool,
}

pub async fn handle_list(options: ListOptions, state: State) -> Result<(), std::io::Error> {
    let projects = state
        .ctx
        .me
        .expect("You are not logged in. Please run `hop auth login` first.")
        .projects;

    if options.quiet {
        let ids = projects
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{}", ids);
    } else {
        let projects_fmt = format_projects(&projects, &state.ctx.default_project, true);

        println!("{}", projects_fmt.join("\n"));
    }

    Ok(())
}
