use std::io::Write;

use anyhow::{anyhow, Result};
use serde_json::Value;
use tabwriter::TabWriter;

use super::types::{Channel, ChannelType, CreateChannel, MultipleChannels, SingleChannel};
use crate::state::http::HttpClient;

pub async fn create_channel(
    http: &HttpClient,
    project_id: &str,
    type_: &ChannelType,
    state: &Value,
    cutsom_id: Option<&str>,
) -> Result<Channel> {
    let (method, path) = match cutsom_id {
        Some(id) => ("PUT", format!("/channels/{}", id)),
        None => ("POST", "/channels".to_string()),
    };

    let response = http
        .request::<SingleChannel>(
            method,
            &format!("{path}?project={project_id}"),
            Some((
                serde_json::to_vec(&CreateChannel {
                    type_: type_.clone(),
                    state: state.clone(),
                })?
                .into(),
                "application/json",
            )),
        )
        .await?
        .ok_or_else(|| anyhow!("Error while parsing response"))?;

    Ok(response.channel)
}

pub async fn get_all_channels(http: &HttpClient, project_id: &str) -> Result<Vec<Channel>> {
    let response = http
        .request::<MultipleChannels>("GET", &format!("/channels?project={}", project_id), None)
        .await?
        .ok_or_else(|| anyhow!("Error while parsing response"))?;

    Ok(response.channels)
}

pub async fn delete_channel(http: &HttpClient, project_id: &str, channel_id: &str) -> Result<()> {
    http.request::<Value>(
        "DELETE",
        &format!("/channels/{channel_id}?project={project_id}"),
        None,
    )
    .await?;

    Ok(())
}

pub fn format_channels(log: &[Channel], title: bool) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(tw, "ID\tTYPE\tSTATE\tCREATED AT").unwrap();
    }

    for channel in log {
        writeln!(
            tw,
            "{}\t{}\t{}\t{}",
            channel.id, channel.type_, channel.state, channel.created_at
        )
        .unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}
