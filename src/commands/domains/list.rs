use anyhow::Result;
use clap::Parser;

use super::util::{format_domains, get_all_domains};
use crate::commands::gateways::util::{format_gateways, get_all_gateways};
use crate::commands::ignite::util::{format_deployments, get_all_deployments};
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List all domains attached to a gateway")]
pub struct Options {
    #[clap(name = "gateway", help = "ID of the gateway")]
    pub gateway: Option<String>,

    #[clap(short = 'q', long = "quiet", help = "Only display domain IDs")]
    pub quiet: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let gateway_id = match options.gateway {
        Some(id) => id,

        None => {
            let project_id = state.ctx.current_project_error().id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;
            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact_opt()
                .expect("Failed to select deployment")
                .expect("No deployment selected");

            let gateways = get_all_gateways(&state.http, &deployments[idx].id).await?;
            let gateways_fmt = format_gateways(&gateways, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a gateway")
                .default(0)
                .items(&gateways_fmt)
                .interact_opt()?
                .expect("No gateways selected");

            gateways[idx].id.clone()
        }
    };

    let domains = get_all_domains(&state.http, &gateway_id).await?;

    if options.quiet {
        let ids = domains
            .iter()
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{}", ids);
    } else {
        let domains_fmt = format_domains(&domains, true);

        println!("{}", domains_fmt.join("\n"));
    }

    Ok(())
}
