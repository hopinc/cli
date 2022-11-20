use std::convert::Infallible;

use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use tokio::sync::mpsc::{channel, Sender};
use tokio::task;

use super::WEB_AUTH_URL;
use crate::commands::auth::login::PAT_FALLBACK_URL;
use crate::utils::parse_key_val;

pub async fn browser_login() -> String {
    let port = portpicker::pick_unused_port().unwrap();

    let callback_url = format!("http://localhost:{}/", port);
    let auth_url = format!(
        "{}?{}",
        WEB_AUTH_URL,
        vec!["callback", &callback_url].join("=")
    );

    // lunch a web server to handle the auth request
    if webbrowser::open(&auth_url).is_ok() {
        log::info!("Opening browser to: {}", auth_url);

        web_auth(port)
            .await
            .expect("Error while starting web auth server")
    } else {
        log::info!("Could not open web a browser.");
        log::info!("Please provide a personal access token manually.");
        log::info!("You can create one at {}", PAT_FALLBACK_URL);

        // falback to simpe input
        dialoguer::Password::new()
            .with_prompt("Enter your token")
            .interact()
            .expect("Failed to get token")
    }
}

async fn web_auth(port: u16) -> Result<String> {
    let (sender, mut receiver) = channel::<String>(1);

    let timeouter = sender.clone();

    let timeout = task::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;
        timeouter.send("timeout".to_string()).await.unwrap();
    });

    let service = make_service_fn(move |_| {
        let sender = sender.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                request_handler(req, sender.clone())
            }))
        }
    });

    let address = ([127, 0, 0, 1], port).into();

    let server = Server::bind(&address).serve(service);

    let runtime = task::spawn(async move {
        if let Err(error) = server.await {
            eprintln!("Server error: {}", error);
        }
        timeout.abort();
    });

    let response = receiver.recv().await;

    runtime.abort();

    assert!(
        Some("timeout".to_string()) != response,
        "Reached the 2 minute timeout"
    );

    Ok(response.unwrap())
}

async fn request_handler(req: Request<Body>, sender: Sender<String>) -> Result<Response<Body>> {
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
