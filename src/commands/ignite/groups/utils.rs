use std::{io::Write, str::FromStr};

use anyhow::{anyhow, ensure, Error, Ok, Result};
use chrono::{DateTime, Utc};
use hop::ignite::groups::types::Group;
use tabwriter::TabWriter;

use crate::{
    commands::ignite::{types::Deployment, utils::get_all_deployments},
    state::State,
    utils::relative_time,
};

pub fn format_groups(groups: &[Group]) -> Result<Vec<String>> {
    let mut tw = TabWriter::new(vec![]);

    for group in groups {
        writeln!(
            tw,
            "{}\t({}) - {}",
            group.name,
            group.id,
            relative_time(group.created_at)
        )?;
    }

    Ok(String::from_utf8(tw.into_inner()?)?
        .lines()
        .map(std::string::ToString::to_string)
        .collect())
}

const BOTTOM_RIGHT: char = '├';
const UP_RIGHT: char = '└';
const HORIZONTAL: char = '─';
const MIN_LINE_PAD: usize = 4;

pub fn format_grouped_deployments_and_order(
    groups: &[Group],
    deployments: &[Deployment],
    details: bool,
    compact: bool,
) -> Result<(Vec<String>, Vec<Deployment>, Vec<usize>)> {
    let mut tw = TabWriter::new(vec![]);
    let mut deployments_ord = vec![];
    let mut ignore_list = vec![];

    if !groups.is_empty() {
        let horiz_pad = (groups
            .iter()
            .map(|group| group.name.len())
            .max()
            .unwrap_or_default()
            / 2
            + 1)
        .min(MIN_LINE_PAD);

        for group in groups {
            let deployments = deployments
                .iter()
                .filter(|d| d.group_id == Some(group.id.clone()));

            let last_idx = deployments.clone().count() - 1;

            for (idx, deployment) in deployments.enumerate() {
                deployments_ord.push(deployment.to_owned());

                match idx {
                    _ if idx == last_idx => {
                        writeln!(
                            tw,
                            "{}",
                            format_deployment(
                                deployment,
                                details,
                                Some(&pad(&UP_RIGHT.to_string(), horiz_pad))
                            )?,
                        )?;
                    }
                    idx => {
                        if idx == 0usize {
                            writeln!(tw, "{}", group.name)?;
                            ignore_list.push(deployments_ord.len() - 1 + ignore_list.len());
                        }

                        writeln!(
                            tw,
                            "{}",
                            format_deployment(
                                deployment,
                                details,
                                Some(&pad(&BOTTOM_RIGHT.to_string(), horiz_pad))
                            )?,
                        )?;
                    }
                }
            }

            if !compact {
                writeln!(tw)?;
            }
        }
    }

    for deployment in deployments.iter().filter(|dep| dep.group_id.is_none()) {
        deployments_ord.push(deployment.to_owned());
        writeln!(tw, "{}", format_deployment(deployment, details, None)?)?;
    }

    Ok((
        String::from_utf8(tw.into_inner()?)?
            .lines()
            .map(std::string::ToString::to_string)
            .collect(),
        deployments_ord,
        ignore_list,
    ))
}

/// Add horizontal lines to the end of a string until it reaches the specified length
fn pad(s: &str, len: usize) -> String {
    let mut s = s.to_string();

    while s.chars().count() < len {
        s.push(HORIZONTAL)
    }

    s
}

fn format_deployment(
    deployment: &Deployment,
    details: bool,
    prefix: Option<&str>,
) -> Result<String> {
    Ok(format!(
        "{}{}\t{}/{}\t{}\t{}",
        if let Some(prefix) = prefix {
            format!("{} ", prefix)
        } else {
            "".to_string()
        },
        if !details {
            deployment.name.clone()
        } else {
            format!("{}\t({})", deployment.name, deployment.id)
        },
        deployment.container_count,
        deployment.target_container_count,
        relative_time(DateTime::<Utc>::from_str(&deployment.created_at)?),
        deployment.config.type_,
    ))
}

pub async fn fetch_grouped_deployments(
    state: &State,
    details: bool,
    compact: bool,
) -> Result<(
    Vec<String>,
    Vec<Deployment>,
    impl Fn(usize) -> Result<usize, Error>,
)> {
    let project_id = state.ctx.current_project_error()?.id;

    let (groups, deployments) = tokio::join!(
        state.hop.ignite.groups.get_all(&project_id),
        get_all_deployments(&state.http, &project_id)
    );

    let (groups, deployments) = (groups?, deployments?);

    ensure!(!deployments.is_empty(), "No deployments found");

    let start = tokio::time::Instant::now();

    let (fmt, deps, ignore_list) =
        format_grouped_deployments_and_order(&groups, &deployments, details, compact)?;

    log::debug!(
        "format_grouped_deployments_and_order took {:?}",
        start.elapsed()
    );

    // write a closure that will return an error if the index is in the ignore list
    let closure = move |idx| {
        let search = ignore_list.binary_search(&idx);

        // because indexes are ordered, we can subtract the possible index from the current index
        // to get the actual index in the `deps` vector
        if let Err(possible) = search {
            Ok(idx - possible)
        } else {
            Err(anyhow!("Invalid index selected"))
        }
    };

    Ok((fmt, deps, closure))
}
