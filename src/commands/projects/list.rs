use structopt::StructOpt;

use crate::{commands::projects::util::format_projects, state::State};

#[derive(Debug, StructOpt)]
#[structopt(about = "List all projects")]
pub struct ListOptions {}

pub async fn handle_list(_options: ListOptions, state: State) -> Result<(), std::io::Error> {
    let projects = state
        .ctx
        .me
        .expect("You are not logged in. Please run `hop auth login` first.")
        .projects;

    if projects.is_empty() {
        panic!("No projects found");
    }

    let projects_fmt = format_projects(&projects, &state.ctx.default_project);

    println!("Projects:");
    println!("{}", projects_fmt.join("\n"));

    Ok(())
}
