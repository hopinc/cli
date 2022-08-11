use std::{borrow::Borrow, io::Write};

use anyhow::{anyhow, Result};
use console::style;
use serde_json::Value;
use tabwriter::TabWriter;

use super::types::{
    ChangeableContainerState, Container, ContainerState, CreateContainers, Log, LogsResponse,
    MultipleContainersResponse, UpdateContainerState,
};
use crate::{state::http::HttpClient, utils::relative_time};

pub async fn create_containers(
    http: &HttpClient,
    deployment_id: &str,
    count: u64,
) -> Result<Vec<Container>> {
    let response = http
        .request::<MultipleContainersResponse>(
            "POST",
            &format!("/ignite/deployments/{}/containers", deployment_id),
            Some((
                serde_json::to_string(&CreateContainers { count })
                    .unwrap()
                    .into(),
                "application/json",
            )),
        )
        .await?
        .ok_or_else(|| anyhow!("Error while parsing response"))?;

    Ok(response.containers)
}

pub async fn delete_container(http: &HttpClient, container_id: &str) -> Result<()> {
    http.request::<()>(
        "DELETE",
        &format!("/ignite/containers/{}", container_id),
        None,
    )
    .await?;

    Ok(())
}

pub async fn get_all_containers(http: &HttpClient, deployment_id: &str) -> Result<Vec<Container>> {
    let response = http
        .request::<MultipleContainersResponse>(
            "GET",
            &format!("/ignite/deployments/{deployment_id}/containers"),
            None,
        )
        .await?
        .ok_or_else(|| anyhow!("Error while parsing response"))?;

    Ok(response.containers)
}

pub async fn update_container_state(
    http: &HttpClient,
    container_id: &str,
    preferred_state: &ChangeableContainerState,
) -> Result<()> {
    http.request::<Value>(
        "PUT",
        &format!("/ignite/containers/{container_id}/state"),
        Some((
            serde_json::to_string(&UpdateContainerState { preferred_state })
                .unwrap()
                .into(),
            "application/json",
        )),
    )
    .await?;

    Ok(())
}

pub async fn get_container_logs(
    http: &HttpClient,
    container_id: &str,
    limit: u64,
    order_by: &str,
) -> Result<Vec<Log>> {
    let response = http
        .request::<LogsResponse>(
            "GET",
            &format!(
                "/ignite/containers/{container_id}/logs?limit={limit}&orderBy={order_by}&offset=0"
            ),
            None,
        )
        .await?
        .ok_or_else(|| anyhow!("Error while parsing response"))?;

    Ok(response.logs)
}

const UNAVAILABLE_ELEMENT: &str = "-";

pub fn format_containers(containers: &Vec<Container>, title: bool) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(tw, "ID\tREGION\tSTATE\tINTERNAL IP\tUPTIME").unwrap();
    }

    for container in containers {
        writeln!(
            tw,
            "{}\t{}\t{}\t{}\t{}",
            container.id,
            container.region,
            container.state,
            container
                .internal_ip
                .as_ref()
                .map(|ip| ip.borrow())
                .unwrap_or_else(|| UNAVAILABLE_ELEMENT),
            if container.state != ContainerState::Running {
                UNAVAILABLE_ELEMENT.to_string()
            } else {
                container
                    .uptime
                    .as_ref()
                    .map(|u| relative_time(u.last_start))
                    .unwrap_or_else(|| UNAVAILABLE_ELEMENT.to_string())
            },
        )
        .unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}

pub fn format_logs(log: &[Log], colors: bool) -> Vec<String> {
    log.iter().map(|log| format_log(log, colors)).collect()
}

fn format_log(log: &Log, colors: bool) -> String {
    let log_level = if colors {
        match log.level.as_str() {
            "info" => style("INFO").cyan(),
            "error" => style("ERROR").red(),
            // there are only info and error, this is left for future use
            level => style(level).yellow(),
        }
        .bold()
        .to_string()
    } else {
        log.level.clone()
    };

    let timestamp = if colors {
        style(log.timestamp.to_rfc2822()).dim().to_string()
    } else {
        log.timestamp.to_rfc2822()
    };

    format!("{timestamp} {log_level} {}", log.message)
}
