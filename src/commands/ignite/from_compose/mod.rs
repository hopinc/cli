mod types;
pub mod utils;

use crate::state::State;
use anyhow::{bail, Result};
use clap::Parser;
use regex::bytes::Regex;
use std::path::{Path, PathBuf};
use types::DockerCompose;

use self::utils::parse_restart_policy;

#[derive(Debug, Parser)]
#[clap(about = "Creates new Ignite deployments from a Docker compose file")]
pub struct Options {
    #[clap(
        name = "file",
        help = "The file to read from. Defaults to docker-compose.yml"
    )]
    pub file: Option<PathBuf>,
}

pub async fn handle(options: Options, _state: State) -> Result<()> {
    let file = match options.file {
        Some(file) => file,
        None => Path::new("docker-compose.yml").to_path_buf(),
    };

    if !file.exists() {
        bail!("File {} does not exist", file.display());
    }

    let compose = std::fs::read(file)?;

    let compose: DockerCompose = match serde_yaml::from_slice(&compose) {
        Ok(compose) => compose,
        Err(error) => {
            log::debug!("Failed to parse compose file: {}", error);

            // note from alistair — I am writing this file as I am learning rust. currently I have no idea
            // how I can implement a custom Deserialize that will provide a better error message
            // including the name of the field that failed to deserialize. So, the code below
            // is just parsing the error string.

            // Reading:
            // https://stackoverflow.com/questions/61107467/is-there-a-way-to-extract-the-missing-field-name-from-serde-jsonerror

            let message = error.to_string();

            let captures = Regex::new(r"unknown field `(.*)`, expected");
            if captures.is_err() {
                bail!(
                    "Failed to parse docker-compose.yml: {}",
                    captures.err().unwrap()
                );
            }

            let captures = captures.unwrap().captures(message.as_bytes());
            if captures.is_none() {
                bail!("Failed to parse error message. Please report this issue!");
            }

            let captures = captures.unwrap();
            let capture = captures.get(1).unwrap().as_bytes();
            let field = std::str::from_utf8(capture).unwrap();

            bail!("Failed to parse Docker compose. The Hop CLI does not currently support the `{}` field", field);
        }
    };

    let services = compose.services.as_ref().unwrap().iter();

    for (name, service) in services {
        let restart_policy = parse_restart_policy(&service.restart).unwrap();
        println!("Restart policy for {} is {}", name, restart_policy)
    }

    Ok(())
}
