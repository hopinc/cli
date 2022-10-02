use std::{io::Write, str::FromStr};

use anyhow::{anyhow, Result};
use chrono::Utc;
use ms::{__to_ms__, ms};
use serde_json::Value;
use tabwriter::TabWriter;

use super::types::{CreateLeapToken, LeapToken, MultipleLeapToken, SingleLeapToken};
use crate::state::http::HttpClient;

pub async fn create_token(
    http: &HttpClient,
    project_id: &str,
    expires_at: Option<&str>,
    state: Option<Value>,
) -> Result<LeapToken> {
    let data = serde_json::to_vec(&CreateLeapToken {
        expires_at: expires_at.map(|s| s.to_owned()),
        state,
    })?;

    let response = http
        .request::<SingleLeapToken>(
            "POST",
            &format!("/channels/tokens?project={project_id}"),
            Some((data.into(), "application/json")),
        )
        .await?
        .ok_or_else(|| anyhow!("Error while parsing response"))?;

    Ok(response.token)
}

pub async fn delete_token(http: &HttpClient, project_id: &str, token: &str) -> Result<()> {
    http.request::<()>(
        "DELETE",
        &format!("/channels/tokens/{token}?project={project_id}"),
        None,
    )
    .await?;

    Ok(())
}

pub async fn get_all_tokens(http: &HttpClient, project_id: &str) -> Result<Vec<LeapToken>> {
    let response = http
        .request::<MultipleLeapToken>(
            "GET",
            &format!("/channels/tokens?project={project_id}"),
            None,
        )
        .await?
        .ok_or_else(|| anyhow!("Error while parsing response"))?;

    Ok(response.tokens)
}

pub fn format_tokens(tokens: &[LeapToken], title: bool) -> Vec<String> {
    let mut tw = TabWriter::new(vec![]);

    if title {
        writeln!(tw, "ID\tSTATE\tCREATION\tEXPIRATION").unwrap();
    }

    for channel in tokens {
        writeln!(
            tw,
            "{}\t{}\t{}\t{}",
            channel.id,
            channel
                .state
                .as_ref()
                .map(|state| state.to_string())
                .unwrap_or_else(|| "null".to_owned()),
            channel.created_at,
            channel.expires_at.as_ref().unwrap_or(&"-".to_owned()),
        )
        .unwrap();
    }

    String::from_utf8(tw.into_inner().unwrap())
        .unwrap()
        .lines()
        .map(std::string::ToString::to_string)
        .collect()
}

pub fn parse_expiration(expires_at: &str) -> Result<String> {
    let now = chrono::Utc::now();

    let date = if expires_at.split('-').count() == 3 || expires_at.split('/').count() == 3 {
        chrono::DateTime::<Utc>::from_str(expires_at).map_err(|_| anyhow!("Invalid date format"))?
    } else {
        let relative = ms!(expires_at).ok_or_else(|| anyhow!("Invalid date format"))?;

        now + chrono::Duration::milliseconds(relative as i64)
    };

    if date < now {
        return Err(anyhow!("Expiration date must be in the future"));
    }

    Ok(date.to_rfc3339())
}
