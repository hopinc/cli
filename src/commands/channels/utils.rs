use std::io::Write;

use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use tabwriter::TabWriter;

use super::types::{
    Channel, ChannelType, CreateChannel, MessageEvent, PaginatedChannels, SingleChannel,
};
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

const PAGE_SIZE: u64 = 75;

pub async fn get_all_channels(http: &HttpClient, project_id: &str) -> Result<Vec<Channel>> {
    let mut channels = vec![];

    let mut page = 1;

    loop {
        let mut paginated = get_channels_in_page(http, project_id, page).await?;

        // this might look annyoing but,
        // to get the correct ordering of the channels
        // we need to reverse the order of the pages
        paginated.channels.extend(channels);
        channels = paginated.channels;

        if paginated.total_count <= channels.len().try_into()? {
            break;
        }

        page += 1;
    }

    Ok(channels)
}

async fn get_channels_in_page(
    http: &HttpClient,
    project_id: &str,
    page: usize,
) -> Result<PaginatedChannels> {
    let response = http
        .request::<PaginatedChannels>(
            "GET",
            &format!(
                "/channels?project={}&page={}&pageSize={PAGE_SIZE}",
                project_id, page
            ),
            None,
        )
        .await?
        .ok_or_else(|| anyhow!("Error while parsing response"))?;

    Ok(response)
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

pub async fn message_channel(
    http: &HttpClient,
    project_id: &str,
    channel_id: &str,
    event: &str,
    data: Option<Value>,
) -> Result<()> {
    http.request::<Value>(
        "POST",
        &format!("/channels/{channel_id}/messages?project={project_id}"),
        Some((
            serde_json::to_vec(&MessageEvent {
                event: event.to_string(),
                data: data.unwrap_or_else(|| json!({})),
            })?
            .into(),
            "application/json",
        )),
    )
    .await?;

    Ok(())
}

pub async fn subscribe_to_channel(
    http: &HttpClient,
    project: &str,
    channel: &str,
    token: &str,
) -> Result<()> {
    http.request::<Value>(
        "PUT",
        &format!("/channels/{channel}/subscribers/{token}?project={project}"),
        None,
    )
    .await?;

    Ok(())
}

pub fn format_channels(channels: &[Channel], title: bool) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(tw, "ID\tTYPE\tCREATION").unwrap();
    }

    for channel in channels {
        writeln!(
            tw,
            "{}\t{}\t{}",
            channel.id, channel.type_, channel.created_at
        )
        .unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}
