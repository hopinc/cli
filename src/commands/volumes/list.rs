use std::collections::HashMap;
use std::io::Write;

use anyhow::{Context, Result};
use clap::Parser;
use tabwriter::TabWriter;

use crate::{commands::volumes::utils::permission_to_string, state::State};

use super::{types::Files, utils::get_files_for_path};

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
    let mut files_map = HashMap::new();

    let volume = format!(
        "volume_{}",
        options
            .deployment
            .split('_')
            .nth(1)
            .context("Failed to get volume from deployment")?
    );

    let files_to_get = if options.files.is_empty() {
        vec![String::new()]
    } else {
        options.files.clone()
    };

    for file in files_to_get {
        let path = file.strip_prefix('/').unwrap_or(&file);

        files_map.insert(
            path.to_string(),
            get_files_for_path(&state.http, &options.deployment, &volume, path).await?,
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
            Files::Single { file } => {
                if options.long {
                    writeln!(
                        tw,
                        "{}\t{}\t{path}",
                        permission_to_string(file.permissions)?,
                        file.size,
                    )?;
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
                        writeln!(
                            tw,
                            "{}\t{}\t{}",
                            permission_to_string(file.permissions)?,
                            file.size,
                            file.name
                        )?;
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
