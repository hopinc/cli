use std::collections::HashMap;
use std::io::Write;

use anyhow::{bail, Context, Result};
use clap::Parser;
use tabwriter::TabWriter;

use super::types::Files;
use super::utils::{format_file, get_files_for_path};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List information about the FILEs (the current directory by default).")]
pub struct Options {
    #[clap(help = "The deployment to list files from")]
    pub deployment: String,
    #[clap(help = "The path(s) to list files from")]
    pub files: Vec<String>,
    #[clap(short, long, help = "Use a long listing format")]
    pub long: bool,
    #[clap(short, long, help = "Do not ignore entries starting with .")]
    pub all: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment = state
        .get_deployment_by_name_or_id(&options.deployment)
        .await?;

    if !deployment.is_stateful() {
        bail!("Deployment is not stateful");
    }

    let mut files_map = HashMap::new();

    let volume = format!(
        "volume_{}",
        deployment
            .id
            .split('_')
            .nth(1)
            .context("Failed to get volume from deployment")?
    );

    let files_to_get = if options.files.is_empty() {
        vec![String::from("/")]
    } else {
        options.files
    };

    for file in files_to_get {
        files_map.insert(
            file.clone(),
            get_files_for_path(&state.http, &deployment.id, &volume, &file).await?,
        );
    }

    let is_mult_checked = files_map.len() > 1;
    let mut is_first_element = true;

    let mut tw = TabWriter::new(std::io::stdout());

    for (path, files) in files_map {
        if !is_first_element {
            writeln!(tw)?;
        } else {
            is_first_element = false;
        }

        match files {
            Files::Single { mut file } => {
                if options.long {
                    file.name = path;

                    writeln!(tw, "{}", format_file(&file)?)?;
                } else {
                    writeln!(tw, "{path}")?;
                }
            }
            Files::Multiple { mut file } => {
                if is_mult_checked {
                    writeln!(tw, "{path}:")?;
                }

                if !options.all {
                    file.retain(|x| !x.name.starts_with('.'));
                }

                for file in file {
                    if options.long {
                        writeln!(tw, "{}", format_file(&file)?)?;
                    } else {
                        write!(tw, "{}\t", file.name)?;
                    }
                }

                if !options.long {
                    writeln!(tw)?;
                }
            }
        }
    }

    // flush the tabwriter to stdout
    tw.flush()?;

    Ok(())
}
