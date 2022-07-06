use crate::state::State;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "List all available projects")]
pub struct ListOptions {}

pub async fn handle_list(_options: ListOptions, state: State) -> Result<(), std::io::Error> {
    let projects = state
        .ctx
        .me
        .clone()
        .expect("You are not logged in. Please run `hop auth login` first.")
        .projects;

    if projects.len() == 0 {
        panic!("No projects found");
    }

    let projects_fmt = projects
        .iter()
        .map(|p| format!(" {} @{} ({})", p.name, p.namespace, p.id))
        .collect::<Vec<_>>();

    println!("Available projects:");
    println!("{}", projects_fmt.join("\n"));

    Ok(())
}
