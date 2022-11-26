use anyhow::Result;
use clap::Parser;

use crate::commands::projects::utils::{create_project, format_project};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Create a new project")]
pub struct Options {
    #[clap(help = "Namespace of the project")]
    namespace: String,
    #[clap(help = "Name of the project")]
    name: String,
    #[clap(short, long, help = "Set as default project")]
    default: bool,
}

pub async fn handle(options: Options, mut state: State) -> Result<()> {
    let project = create_project(&state.http, &options.name, &options.namespace).await?;

    if options.default {
        state.ctx.default_project = Some(project.id.clone());
        state.ctx.save().await?;
    }

    log::info!("Created project {}", format_project(&project));

    Ok(())
}
