use anyhow::Result;
use clap::Parser;

use crate::commands::gateways::util::{format_gateways, get_all_gateways};
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List all Gateways")]
#[group(skip)]
pub struct Options {
    #[clap(name = "deployment", help = "ID of the deployment")]
    pub deployment: Option<String>,

    #[clap(
        short = 'q',
        long = "quiet",
        help = "Only print the IDs of the deployments"
    )]
    pub quiet: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment_id = match options.deployment {
        Some(deployment) => deployment,

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

            deployments[idx].id.clone()
        }
    };

    let gateways = get_all_gateways(&state.http, &deployment_id).await?;

    if options.quiet {
        let ids = gateways
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{ids}");
    } else {
        let containers_fmt = format_gateways(&gateways, true);

        println!("{}", containers_fmt.join("\n"));
    }

    Ok(())
}
