use std::io::Write;

use anyhow::{ensure, Result};
use clap::Parser;
use tabwriter::TabWriter;

use super::utils::{format_containers, get_all_containers, get_container, UNAVAILABLE_ELEMENT};
use crate::commands::containers::utils::format_single_metrics;
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::commands::ignite::utils::get_deployment;
use crate::state::State;
use crate::utils::relative_time;

#[derive(Debug, Parser)]
#[clap(about = "Inspect a container")]
#[group(skip)]
pub struct Options {
    #[clap(help = "ID of the container")]
    pub container: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let (container, deployment) = if let Some(container_id) = options.container {
        let container = get_container(&state.http, &container_id).await?;
        let deployment = get_deployment(&state.http, &container.deployment_id).await?;

        (container, deployment)
    } else {
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

        let deployment = deployments[idx].to_owned();

        let containers = get_all_containers(&state.http, &deployment.id).await?;
        ensure!(!containers.is_empty(), "No containers found");
        let containers_fmt = format_containers(&containers, false);

        let idx = dialoguer::Select::new()
            .with_prompt("Select container")
            .items(&containers_fmt)
            .default(0)
            .interact()?;

        (containers[idx].to_owned(), deployment)
    };

    let mut tw = TabWriter::new(vec![]);

    writeln!(tw, "{}", container.id)?;
    writeln!(tw, "  Metadata")?;
    writeln!(tw, "\tDeployment: {} ({})", deployment.name, deployment.id)?;
    writeln!(tw, "\tCreated: {} ago", relative_time(container.created_at))?;
    writeln!(tw, "\tState: {}", container.state)?;
    writeln!(
        tw,
        "\tUptime: {}",
        container
            .uptime
            .as_ref()
            .map(|u| {
                u.last_start
                    .map(relative_time)
                    .unwrap_or_else(|| UNAVAILABLE_ELEMENT.to_string())
            })
            .unwrap_or_else(|| UNAVAILABLE_ELEMENT.to_string())
    )?;
    writeln!(
        tw,
        "\tInternal IP: {}",
        container
            .internal_ip
            .unwrap_or_else(|| UNAVAILABLE_ELEMENT.to_string())
    )?;
    writeln!(tw, "\tRegion: {}", container.region)?;
    writeln!(tw, "\tType: {}", container.type_)?;
    writeln!(tw, "  Metrics")?;

    for metric in format_single_metrics(&container.metrics, &deployment)? {
        writeln!(tw, "\t{}", metric)?;
    }

    tw.flush()?;

    print!("{}", String::from_utf8(tw.into_inner()?)?);

    Ok(())
}
