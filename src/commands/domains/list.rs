use anyhow::{ensure, Result};
use clap::Parser;

use super::util::{format_domains, get_all_domains};
use crate::commands::gateways::util::{format_gateways, get_all_gateways};
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "List all domains attached to a Gateway")]
#[group(skip)]
pub struct Options {
    #[clap(help = "ID of the Gateway")]
    pub gateway: Option<String>,

    #[clap(short, long, help = "Only display domain IDs")]
    pub quiet: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let gateway_id = match options.gateway {
        Some(id) => id,

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

            let gateways = get_all_gateways(&state.http, &deployments[idx].id).await?;
            ensure!(!gateways.is_empty(), "No Gateways found");
            let gateways_fmt = format_gateways(&gateways, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a Gateway")
                .default(0)
                .items(&gateways_fmt)
                .interact()?;

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

        println!("{ids}");
    } else {
        let domains_fmt = format_domains(&domains, true);

        println!("{}", domains_fmt.join("\n"));
    }

    Ok(())
}
