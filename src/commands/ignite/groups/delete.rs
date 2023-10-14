use anyhow::{ensure, Result};
use clap::Parser;

use crate::state::State;

use super::utils::format_groups;

#[derive(Debug, Parser)]
#[clap(about = "Delete an Ignite group")]
#[group(skip)]
pub struct Options {
    #[clap(help = "The ID of the group")]
    pub group: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let group = if let Some(group) = options.group {
        group
    } else {
        let project = state.ctx.current_project_error()?;

        let mut groups = state.hop.ignite.groups.get_all(&project.id).await?;

        ensure!(!groups.is_empty(), "No groups found");

        groups.sort_unstable_by_key(|group| group.position);

        let dialoguer_groups = dialoguer::Select::new()
            .with_prompt("Select group")
            .items(&format_groups(&groups)?)
            .interact()?;

        groups[dialoguer_groups].id.clone()
    };

    state.hop.ignite.groups.delete(&group).await?;

    Ok(())
}
