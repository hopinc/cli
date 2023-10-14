use anyhow::Result;
use clap::Parser;

use crate::commands::containers::utils::{format_containers, get_all_containers};
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::commands::ignite::utils::get_deployment;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List all containers")]
#[group(skip)]
pub struct Options {
    #[clap(help = "ID of the deployment")]
    pub deployment: Option<String>,

    #[clap(short, long, help = "Only print the IDs of the deployments")]
    pub quiet: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment = match options.deployment {
        Some(id) => get_deployment(&state.http, &id).await?,

        None => {
            let (deployments_fmt, deployments, validator) =
                fetch_grouped_deployments(&state, false, true).await?;

            let idx = loop {
                let idx = dialoguer::Select::new()
                    .with_prompt("Select a deployment")
                    .items(&deployments_fmt)
                    .default(0)
                    .interact()?;

                if let Ok(idx) = validator(idx) {
                    break idx;
                }

                console::Term::stderr().clear_last_lines(1)?
            };

            deployments[idx].clone()
        }
    };

    let containers = get_all_containers(&state.http, &deployment.id).await?;

    if options.quiet {
        let ids = containers
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{ids}");
    } else {
        let containers_fmt = format_containers(&containers, true);

        println!("{}", containers_fmt.join("\n"));
    }

    Ok(())
}
