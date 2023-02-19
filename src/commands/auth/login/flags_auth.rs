use anyhow::{bail, Context, Result};

use super::types::{KeyType, LoginRequest, LoginResponse, SecondFactorRequest};
use super::Options;
use crate::config::EXEC_NAME;
use crate::state::http::HttpClient;

pub async fn flags_login(options: Options, http: HttpClient) -> Result<String> {
    match options {
        Options {
            token: Some(_) | None,
            email: None,
            password: None,
            ..
        } => {
            if options.token.is_none() || options.token.as_ref().unwrap().is_empty() {
                dialoguer::Password::new()
                    .with_prompt("Token")
                    .interact()
                    .context("Error getting token")
            } else {
                options.token.context("Token is empty")
            }
        }

        Options {
            email: Some(username),
            password: None,
            ..
        }
        | Options {
            email: Some(username),
            password: Some(_),
            ..
        } => {
            let password =
                if options.password.is_none() || options.password.as_ref().unwrap().is_empty() {
                    dialoguer::Password::new()
                        .with_prompt("Password")
                        .interact()
                        .expect("Error getting password")
                } else {
                    options.password.unwrap()
                };

            login_with_credentials(http, username, password).await
        }

        _ => panic!("Invalid login options, run `{EXEC_NAME} auth login --help` for more info",),
    }
}

async fn login_with_credentials(
    http: HttpClient,
    email: String,
    password: String,
) -> Result<String> {
    let res = http
        .request::<LoginResponse>(
            "POST",
            "/auth",
            Some((
                serde_json::to_vec(&LoginRequest { email, password })?.into(),
                "application/json",
            )),
        )
        .await?
        .context("Could not parse response from server")?;

    match res {
        LoginResponse::Success { token } => Ok(token),
        LoginResponse::SecondFactorRequired {
            ticket,
            preferred_type,
            mut types,
        } => {
            // sort by user preference
            types.sort_by(|a, b| {
                if a == &preferred_type {
                    std::cmp::Ordering::Less
                } else if b == &preferred_type {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            });

            for key_type in types {
                match key_type {
                    KeyType::Totp => {
                        let code = dialoguer::Input::new()
                            .with_prompt("Please enter your TOTP code")
                            .interact()
                            .context("Error getting second factor code")?;

                        let res = http
                            .request::<LoginResponse>(
                                "POST",
                                "/auth/mfa",
                                Some((
                                    serde_json::to_vec(&SecondFactorRequest::Totp {
                                        code,
                                        ticket,
                                    })?
                                    .into(),
                                    "application/json",
                                )),
                            )
                            .await?
                            .context("Could not parse response from server")?;

                        match res {
                            LoginResponse::Success { token } => return Ok(token),
                            _ => bail!("Invalid second factor response"),
                        }
                    }

                    KeyType::Key => {
                        log::warn!("Key second factor is not supported yet, skipping");
                    }
                }
            }

            bail!("No second factor method was successful")
        }
    }
}
