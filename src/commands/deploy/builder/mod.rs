mod types;
mod util;

use std::path::PathBuf;

use anyhow::{bail, Result};
use hop_leap::leap::types::Event;
use hop_leap::{LeapEdge, LeapOptions};
use tokio::sync::mpsc::unbounded_channel;
use tokio::{fs, spawn};

use self::types::BuildEvents;
use self::util::{builder_post, cancel_build, compress};
use crate::config::HOP_LEAP_PROJECT;
use crate::state::State;
use crate::util::urlify;

pub async fn build(
    state: &State,
    project_id: &str,
    deployment_id: &str,
    dir: PathBuf,
) -> Result<()> {
    // connect to leap here so no logs interfere with the deploy
    let mut leap = LeapEdge::new(LeapOptions {
        token: Some(&state.ctx.current.clone().unwrap().leap_token),
        project: HOP_LEAP_PROJECT,
        ..Default::default()
    })
    .await?;

    leap.channel_subscribe(project_id).await?;

    // deployment id is used not to colide if the user is deploying multiple items
    let packed = compress(deployment_id, dir).await?;

    log::info!("Packed to: {packed}");

    let bytes = fs::read(packed.clone()).await?;

    log::info!("Uploading...");

    let build = builder_post(&state.http, deployment_id, bytes).await?;

    let (tx, mut rx) = unbounded_channel();

    let http = state.http.clone();
    let build_id = build.id.clone();

    spawn(async move {
        loop {
            match rx.recv().await {
                Some("CANCEL") => {
                    log::info!("Cancelling build...");

                    if cancel_build(&http, &build_id).await.is_ok() {
                        log::info!("Build cancelled by user");
                    } else {
                        log::error!("Failed to cancel build");
                    }

                    std::process::exit(1);
                }

                Some("OK") => break,

                _ => {}
            }
        }
    });

    let ctrlc = tx.clone();

    ctrlc::set_handler(move || {
        ctrlc.send("CANCEL").ok();
    })?;

    log::info!("Deleting archive...");
    fs::remove_file(packed).await?;

    log::info!("From Hop builder:");

    while let Some(event) = leap.listen().await {
        if let Event::Message(capsuled) = event {
            if Some(project_id) != capsuled.channel.as_deref() {
                continue;
            }

            let build_data =
                match serde_json::from_value(serde_json::to_value(capsuled.data).unwrap()) {
                    Ok(build_data) => build_data,
                    Err(_) => {
                        // silently ignore

                        continue;
                    }
                };

            match build_data {
                BuildEvents::BuildProgress(build_progress) => {
                    if build_progress.build_id == build.id {
                        print!("{}", build_progress.log);
                    }
                }

                BuildEvents::BuildCancelled(build_cancelled) => {
                    if build_cancelled.build_id == build.id {
                        tx.send("OK").ok();
                        leap.close().await;

                        bail!("Build cancelled");
                    }
                }

                BuildEvents::PushSuccess(build_complete) => {
                    if build_complete.build_id == build.id {
                        tx.send("OK").ok();
                        leap.close().await;

                        println!();

                        log::info!("Build complete");
                    }
                }

                BuildEvents::PushFailure(build_failure) => {
                    if build_failure.build_id == build.id {
                        leap.close().await;

                        println!();

                        bail!(
                                "Push failed, for help contact us on {} and mention the deployment id: {}",
                                urlify("https://discord.gg/hop"),
                                deployment_id
                            );
                    }
                }
            }
        }
    }

    tx.send("OK").ok();

    Ok(())
}
