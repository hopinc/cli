use std::str::FromStr;

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;

use crate::commands::auth::types::{AuthorizedClient, UserMe};
use crate::commands::projects::types::ThisProjectResponse;
use crate::state::http::HttpClient;

#[derive(Debug, Deserialize, Clone)]
pub enum TokenType {
    #[serde(rename = "PAT")]
    Pat,
    #[serde(rename = "PTK")]
    Ptk,
    #[serde(rename = "BEARER")]
    Bearer,
}

impl FromStr for TokenType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        serde_json::from_str(&format!("\"{}\"", s.to_uppercase()))
            .map_err(|_| anyhow!("Could not parse token type: {}", s))
    }
}

impl TokenType {
    pub fn from_token(token: &str) -> Result<Self> {
        Self::from_str(&token.split('_').next().unwrap_or("").to_uppercase())
    }
}

pub async fn token_options(
    http: HttpClient,
    token_type: Option<TokenType>,
) -> Result<AuthorizedClient> {
    match token_type {
        // bearer token works the same as pat
        Some(TokenType::Pat | TokenType::Bearer) => login_pat(http.clone()).await,

        // ptks only allow one project at a time so diff route
        Some(TokenType::Ptk) => login_ptk(http.clone()).await,
        // should be impossible to get here
        token => {
            bail!("Unsupported token type: {token:?}");
        }
    }
}

async fn login_pat(http: HttpClient) -> Result<AuthorizedClient> {
    let response = http
        .request::<UserMe>("GET", "/users/@me", None)
        .await?
        .context("Error while parsing response")?;

    Ok(AuthorizedClient {
        id: response.user.id,
        name: response.user.name,
        projects: response.projects,
        leap_token: response.leap_token,
        email: response.user.email,
        email_verified: response.user.email_verified,
    })
}

async fn login_ptk(http: HttpClient) -> Result<AuthorizedClient> {
    let ThisProjectResponse {
        leap_token,
        project,
    } = http
        .request::<ThisProjectResponse>("GET", "/projects/@this", None)
        .await?
        .context("Error while parsing response")?;

    Ok(AuthorizedClient {
        projects: vec![project.clone()],
        name: project.name,
        id: project.id,
        leap_token,
        email: "user@hop.io".to_string(),
        email_verified: true,
    })
}
