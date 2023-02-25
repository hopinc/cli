use std::env::temp_dir;

use anyhow::{ensure, Result};
use clap::Parser;
use futures_util::StreamExt;
use tokio::fs;
use tokio::process::Command;

use super::utils::{format_containers, format_logs, get_all_containers, get_container_logs};
use crate::commands::ignite::utils::{format_deployments, get_all_deployments};
use crate::config::DEFAULT_EDITOR;
use crate::state::State;
use crate::utils::arisu::{ArisuClient, ArisuMessage};
use crate::utils::in_path;

#[derive(Debug, Parser)]
#[clap(about = "Get logs of a container")]
pub struct Options {
    #[clap(help = "ID of the container")]
    container: Option<String>,

    #[clap(short, long, help = "Follow the logs")]
    follow: bool,

    #[clap(
        short = 'n',
        long,
        help = "Number of lines to show",
        default_value = "10"
    )]
    lines: u64,

    #[clap(short, long, help = "Show the newest entries first")]
    reverse: bool,

    #[clap(short, long, help = "Show timestamps")]
    timestamps: bool,

    #[clap(short, long, help = "Show details")]
    details: bool,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let container = match options.container {
        Some(id) => id,

        None => {
            let project_id = state.ctx.clone().current_project_error()?.id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;
            ensure!(!deployments.is_empty(), "No deployments found");
            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact()?;

            let containers = get_all_containers(&state.http, &deployments[idx].id).await?;
            ensure!(!containers.is_empty(), "No containers found");
            let containers_fmt = format_containers(&containers, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a container")
                .default(0)
                .items(&containers_fmt)
                .interact()?;

            containers[idx].id.clone()
        }
    };

    // initial logs
    let logs = get_container_logs(
        &state.http,
        &container,
        options.lines,
        // doesnt make sense to follow reversed logs
        if options.reverse && !options.follow {
            "asc"
        } else {
            "desc"
        },
    )
    .await?;

    if !options.follow {
        let temp = temp_dir().join(format!("hop_ignite_logs-{container}.txt"));

        fs::write(
            &temp,
            format_logs(&logs, false, options.timestamps, options.details).join("\n"),
        )
        .await?;

        let editor = if in_path("less").await {
            "less".to_string()
        } else {
            std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| DEFAULT_EDITOR.to_string())
        };

        log::info!("Opening logs in `{editor}`");

        if let Err(e) = Command::new(editor).arg(&temp).status().await {
            log::warn!("Failed to open logs: {}", e);
        }

        fs::remove_file(&temp).await?;

        return Ok(());
    }

    println!(
        "{}",
        format_logs(&logs, true, options.timestamps, options.details).join("\n")
    );

    let token = state.token().unwrap();

    let mut arisu = ArisuClient::new(&container, &token).await?;

    while let Some(message) = arisu.next().await {
        match message {
            ArisuMessage::ServiceMessage(data) => log::info!("Service: {data}"),
            ArisuMessage::Out(log) => {
                print!(
                    "{}",
                    format_logs(&[log], true, options.timestamps, options.details)[0]
                );
            }
        }
    }

    Ok(())
}
