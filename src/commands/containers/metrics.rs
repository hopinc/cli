use std::io::Write;

use anyhow::{ensure, Context, Result};
use clap::Parser;
use console::Term;
use futures_util::StreamExt;

use super::utils::{format_containers, get_all_containers, get_container};
use crate::commands::containers::utils::format_single_metrics;
use crate::commands::ignite::groups::utils::fetch_grouped_deployments;
use crate::commands::ignite::utils::get_deployment;
use crate::state::State;
use crate::utils::arisu::{ArisuClient, ArisuMessage};

#[derive(Debug, Parser)]
#[clap(about = "Get metrics for a container")]
#[group(skip)]
pub struct Options {
    #[clap(help = "ID of the container")]
    pub container: Option<String>,

    #[clap(short, long, help = "Show metrics in real time")]
    pub follow: bool,
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

    let mut term = Term::stdout();

    writeln!(
        term,
        "{}",
        format_single_metrics(&container.metrics, &deployment)?.join("\n")
    )?;

    if !options.follow {
        return Ok(());
    }

    let token = state.token().context("No token found")?;

    let mut arisu = ArisuClient::new(&container.id, &token).await?;

    while let Some(message) = arisu.next().await {
        match message {
            ArisuMessage::Open => arisu.request_metrics().await?,

            ArisuMessage::Metrics(metrics) => {
                let metrics = format_single_metrics(&Some(metrics), &deployment)?;

                if !state.debug {
                    term.clear_last_lines(metrics.len())?;
                }

                writeln!(term, "{}", metrics.join("\n"))?
            }

            _ => {}
        }
    }

    Ok(())
}
