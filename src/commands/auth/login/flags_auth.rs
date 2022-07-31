use super::types::{LoginRequest, LoginResponse};
use super::LoginOptions;
use crate::{config::EXEC_NAME, state::http::HttpClient};

pub async fn flags_login(options: LoginOptions, http: HttpClient) -> String {
    match options {
        LoginOptions {
            token: Some(token), ..
        } => token,

        LoginOptions {
            email: Some(username),
            password: Some(password),
            ..
        } => login_with_credentials(http, username, password).await,

        LoginOptions {
            email: Some(username),
            ..
        } => {
            let password = dialoguer::Password::new()
                .with_prompt("Password")
                .interact()
                .ok()
                .expect("Error getting password");

            login_with_credentials(http, username, password).await
        }
        _ => panic!(
            "Invalid login options, run `{} auth login --help` for more info",
            EXEC_NAME
        ),
    }
}

async fn login_with_credentials(http: HttpClient, email: String, password: String) -> String {
    http.request::<LoginResponse>(
        "POST",
        "/auth",
        Some((
            serde_json::to_string(&LoginRequest { email, password })
                .unwrap()
                .into(),
            "application/json",
        )),
    )
    .await
    .expect("Error while logging in")
    .expect("Error while parsing login response")
    .token
}
