use anyhow::Result;
use clap::Parser;

use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Create a ne Ignite group")]
#[group(skip)]
pub struct Options {
    #[clap(help = "The name of the group")]
    pub name: Option<String>,
    #[clap(short, long, help = "The deployments to add to the group")]
    pub deployments: Vec<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let project = state.ctx.current_project_error()?;

    let name = if let Some(name) = options.name {
        name
    } else {
        dialoguer::Input::new()
            .with_prompt("Group Name")
            .interact_text()?
    };

    let deployments = if !options.deployments.is_empty() {
        options.deployments
    } else {
        let (deployments_fmt, deployments, validator) =
            fetch_grouped_deployments(&state, false, true).await?;

        let idxs = loop {
            let idxs = dialoguer::MultiSelect::new()
                .with_prompt("Select deployments")
                .items(&deployments_fmt)
                .interact()?;

            if !idxs.is_empty() && idxs.iter().all(|idx| validator(*idx).is_ok()) {
                break idxs;
            }

            console::Term::stderr().clear_last_lines(1)?
        }
        .into_iter()
        .map(|idx| validator(idx).unwrap())
        .collect::<Vec<_>>();

        idxs.into_iter()
            .map(|idx| deployments[idx].id.clone())
            .collect()
    };

    let group = state
        .hop
        .ignite
        .groups
        .create(
            &project.id,
            &name,
            &deployments.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
        )
        .await?;

    log::info!("Group successfully created. ID: {}\n", group.id);

    Ok(())
}
