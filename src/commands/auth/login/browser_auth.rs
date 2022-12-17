use std::convert::Infallible;

use anyhow::{anyhow, Context, Result};
use hyper::{Body, Request, Response};
use tokio::sync::mpsc::Sender;

use super::WEB_AUTH_URL;
use crate::commands::auth::login::PAT_FALLBACK_URL;
use crate::utils::browser::listen_for_callback;
use crate::utils::parse_key_val;

pub async fn browser_login() -> Result<String> {
    let port = portpicker::pick_unused_port().with_context(|| {
        "Could not find an unused port. Please make sure you have at least one port available."
    })?;

    let url = format!(
        "{WEB_AUTH_URL}?{}",
        vec!["callback", &format!("http://localhost:{port}/")].join("=")
    );

    // lunch a web server to handle the auth request
    if let Err(why) = webbrowser::open(&url) {
        log::error!("Could not open web a browser.");
        log::debug!("Error: {why}");
        log::info!("Please provide a personal access token manually.");
        log::info!("You can create one at {PAT_FALLBACK_URL}");

        // fallback to simple input
        dialoguer::Password::new()
            .with_prompt("Enter your token")
            .interact()
            .map_err(|why| anyhow!(why))
    } else {
        log::info!("Waiting for token to be created...");

        listen_for_callback(port, 2, |req, sender| {
            Box::pin(request_handler(req, sender))
        })
        .await
        .map_err(|why| anyhow!(why))
    }
}

async fn request_handler(
    req: Request<Body>,
    sender: Sender<String>,
) -> Result<Response<Body>, Infallible> {
    let query = req.uri().query();

    // only send if it's an actual token
    if let Some(query) = query {
        // parse the query
        // since pat should be a URL safe string we can just split on '='
        let query: Vec<(String, String)> = query
            .split('&')
            .map(|s| parse_key_val(s).unwrap())
            .collect::<Vec<_>>();

        // if query has a key called "token"
        if let Some(token) = query.iter().find(|(k, _)| *k == "token") {
            // send it to the main thread
            sender.send(token.1.to_string()).await.unwrap();
            return Ok(Response::new("You've been authorized".into()));
        }
    }

    Ok(Response::builder()
        .status(400)
        .body("You're not authorized".into())
        .unwrap())
}
