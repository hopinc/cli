use std::collections::HashMap;
use std::io::Write;

use anyhow::Result;
use clap::Parser;
use tabwriter::TabWriter;

use super::types::Files;
use super::utils::{format_file, get_files_for_path, parse_target_from_path_like};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List information about files")]
#[group(skip)]
pub struct Options {
    #[clap(
        help = "The path(s) to list, in the format <deployment name or id>:<path>",
        required = true
    )]
    pub paths: Vec<String>,
    #[clap(short, long, help = "Use a long listing format")]
    pub long: bool,
    #[clap(short, long, help = "Do not ignore entries starting with .")]
    pub all: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let mut files_map = HashMap::new();

    for file in options.paths {
        let target = parse_target_from_path_like(&state, &file).await?;

        let (deployment, volume, path) = match target {
            (Some((deployment, volume)), path) => (deployment, volume, path),
            (None, _) => {
                log::warn!("No deployment identifier found in `{file}`, skipping, make sure to use the format <deployment name or id>:<path>");

                continue;
            }
        };

        files_map.insert(
            file.clone(),
            get_files_for_path(&state.http, &deployment.id, &volume, &path).await?,
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
