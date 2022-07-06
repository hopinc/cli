use crate::state::State;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Get information about a project")]
pub struct InfoOptions {}

pub async fn handle_command(_options: InfoOptions, state: State) -> Result<(), std::io::Error> {
    if let Some(project) = state.ctx.current_project() {
        println!(
            "Project: `{}` ({}) {}",
            project.name, project.id, project.p_type
        );
    } else {
        println!("No project selected. Run `hop project switch` to select one or use `--project` to specify a project");
    }

    Ok(())
}
