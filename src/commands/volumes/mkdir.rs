use anyhow::Result;
use clap::Parser;

use super::utils::parse_target_from_path_like;
use crate::commands::volumes::utils::create_directory;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete files")]
#[group(skip)]
pub struct Options {
    #[clap(
        help = "The path(s) to delete, in the format <deployment name or id>:<path>",
        required = true
    )]
    pub paths: Vec<String>,

    #[clap(
        short,
        long,
        help = "Create recursive directories if they do not exist"
    )]
    pub recursive: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    for file in options.paths {
        let target = parse_target_from_path_like(&state, &file).await?;

        match target {
            (Some((deployment, volume)), path) => {
                create_directory(
                    &state.http,
                    &deployment.id,
                    &volume,
                    &path,
                    options.recursive,
                )
                .await?;

                log::info!("Created directory `{file}`");
            }

            (None, _) => {
                log::warn!("No deployment identifier found in `{file}`, skipping");
            }
        }
    }

    Ok(())
}
