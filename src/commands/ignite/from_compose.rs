use std::path::Path;

use anyhow::{Error, Result};
use clap::Parser;

use crate::{docker::types::DockerCompose, state::State};

#[derive(Debug, Parser)]
#[clap(about = "Creates new Ignite deployments from a Docker compose file")]
pub struct Options {
    #[clap(
        name = "file",
        help = "The file to read from. Defaults to docker-compose.yml"
    )]
    pub file: Option<String>,
}

pub async fn handle(options: Options, _state: State) -> Result<()> {
    let default_file = "docker-compose.yml".to_owned();
    let file = options.file.unwrap_or(default_file);

    let path = Path::new(&file);

    if !path.exists() {
        return Err(Error::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File {} does not exist", file),
        )));
    }

    let compose = std::fs::read_to_string(path)?;
    let compose: DockerCompose = serde_yaml::from_str(&compose)?;

    println!("{:#?}", compose);

    Ok(())
}
