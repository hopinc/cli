use std::io::Write;

use anyhow::Result;
use ms::{__to_string__, ms};
use serde_json::Value;

use super::types::{Build, MultipleBuilds};
use crate::state::http::HttpClient;
use crate::utils::relative_time;

pub async fn get_all_builds(http: &HttpClient, deployment_id: &str) -> Result<Vec<Build>> {
    let mut response = http
        .request::<MultipleBuilds>(
            "GET",
            &format!("/ignite/deployments/{deployment_id}/builds"),
            None,
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("Could not parse response"))?;

    response
        .builds
        .sort_by_cached_key(|build| std::cmp::Reverse(build.started_at.timestamp()));

    Ok(response.builds)
}

pub async fn cancel_build(http: &HttpClient, build_id: &str) -> Result<()> {
    http.request::<Value>("POST", &format!("/ignite/builds/{build_id}/cancel"), None)
        .await?;

    Ok(())
}

pub fn format_builds(builds: &[Build], title: bool) -> Vec<String> {
    let mut tw = tabwriter::TabWriter::new(vec![]);

    if title {
        writeln!(&mut tw, "ID\tSTATUS\tDIGEST\tMETHOD\tSTARTED\tDURATION").unwrap();
    }

    for build in builds {
        writeln!(
            &mut tw,
            "{}\t{}\t{}\t{}\t{}\t{}",
            build.id,
            build.state,
            build.digest.clone().unwrap_or_else(|| "-".to_string()),
            build.method,
            relative_time(build.started_at),
            build
                .finished_at
                .map(|t| ms!(
                    (t - build.started_at).num_milliseconds().unsigned_abs(),
                    true
                ))
                .unwrap_or_default(),
        )
        .unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}
