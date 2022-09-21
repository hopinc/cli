use anyhow::Result;
use clap::Parser;

use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Creates new Ignite deployments from a Docker compose file")]
pub struct Options {
    #[clap(
        name = "file",
        help = "The file to read from. Defaults to docker-compose.yml"
    )]
    pub file: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let default_file = "docker-compose.yml".to_owned();
    let file = options.file.unwrap_or(default_file);

    Ok(())
}
