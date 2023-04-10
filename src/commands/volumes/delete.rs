use anyhow::Result;
use clap::Parser;

use super::utils::{delete_files_for_path, parse_target_from_path_like, path_into_uri_safe};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Delete files")]
pub struct Options {
    #[clap(
        help = "The path(s) to delete, in the format <deployment name or id>:<path>",
        required = true
    )]
    pub paths: Vec<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    for file in options.paths {
        let target = parse_target_from_path_like(&state, &file).await?;

        match target {
            (Some((deployment, volume)), path) => {
                let path = path_into_uri_safe(&path);

                delete_files_for_path(&state.http, &deployment.id, &volume, &path).await?;

                log::info!("Deleted `{file}`");
            }

            (None, _) => {
                log::warn!("No deployment identifier found in `{file}`, skipping");
            }
        }
    }

    Ok(())
}
