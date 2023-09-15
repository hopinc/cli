use anyhow::{bail, Result};
use clap::Parser;

use crate::state::State;

use super::utils::{move_file, parse_target_from_path_like};

#[derive(Debug, Parser)]
#[clap(about = "List information about files")]
#[group(skip)]
pub struct Options {
    #[clap(help = "The path to move")]
    pub source: String,
    #[clap(help = "The path to move to")]
    pub target: String,
}

/// Handle is writte so it allows:
/// hop volumes mv <deployment name or id>:<path> <deployment name or id>:<path>
/// hop volumes mv <deployment name or id>:<path> <path>
///
/// Which makes it easy for the user, since they don't have to specify the deployment but can
pub async fn handle(options: Options, state: State) -> Result<()> {
    let source = parse_target_from_path_like(&state, &options.source).await?;
    let target = parse_target_from_path_like(&state, &options.target).await?;

    let (deployment, volume) = if let Some((deployment, volume)) = source.0 {
        (deployment, volume)
    } else {
        bail!("No deployment identifier found in `{}`", options.source);
    };

    if let Some((deployment_target, volume_target)) = target.0 {
        if deployment_target.id != deployment.id || volume_target != volume {
            bail!("Cannot move files between deployments");
        }
    }

    move_file(&state.http, &deployment.id, &volume, &source.1, &target.1).await?;

    log::info!("Moved `{}` to `{}`", source.1, target.1);

    Ok(())
}
