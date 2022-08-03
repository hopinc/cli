use std::str::FromStr;

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
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(&format!("\"{}\"", s.to_uppercase())).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Could not parse token type: {}", s),
            )
        })
    }
}

impl TokenType {
    pub fn from_token(token: &str) -> Result<Self, std::io::Error> {
        Self::from_str(&token.split('_').next().unwrap_or("").to_uppercase())
    }
}

pub async fn token_options(http: HttpClient, token_type: Option<TokenType>) -> AuthorizedClient {
    match token_type {
        Some(TokenType::Pat) => login_pat(http.clone()).await,
        // bearer token works the same as pat
        Some(TokenType::Bearer) => login_pat(http.clone()).await,
        // ptks only allow one project at a time so diff route
        Some(TokenType::Ptk) => login_ptk(http.clone()).await,
        // should be impossible to get here
        token => {
            panic!("Unsupported token type: {:?}", token);
        }
    }
}

async fn login_pat(http: HttpClient) -> AuthorizedClient {
    let response = http
        .request::<UserMe>("GET", "/users/@me", None)
        .await
        .expect("Error logging in")
        .expect("Error while parsing response");

    AuthorizedClient {
        id: response.user.id,
        name: response.user.name,
        projects: response.projects,
        leap_token: response.leap_token,
        email: Some(response.user.email),
    }
}

async fn login_ptk(http: HttpClient) -> AuthorizedClient {
    let ThisProjectResponse {
        leap_token,
        project,
    } = http
        .request::<ThisProjectResponse>("GET", "/projects/@this", None)
        .await
        .expect("Error logging in")
        .expect("Error while parsing response");

    AuthorizedClient {
        projects: vec![project.clone()],
        name: project.name,
        id: project.id,
        leap_token,
        email: None,
    }
}
