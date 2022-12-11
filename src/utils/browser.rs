use std::future::Future;
use std::{convert::Infallible, pin::Pin};

use anyhow::{ensure, Result};
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use tokio::sync::mpsc::{channel, Sender};

type RequestHandler =
    fn(
        Request<Body>,
        Sender<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send>>;

pub async fn listen_for_callback(
    port: u16,
    timeout_min: u16,
    request_handler: RequestHandler,
) -> Result<String> {
    let (sender, mut receiver) = channel::<String>(1);

    let timeouter = sender.clone();

    let timeout = tokio::spawn(async move {
        let timeout = timeout_min as u64 * 60;

        tokio::time::sleep(tokio::time::Duration::from_secs(timeout)).await;
        timeouter.send("timeout".to_string()).await.ok();
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

    let runtime = tokio::spawn(async move {
        if let Err(error) = server.await {
            log::error!("Server error: {error}");
        }

        timeout.abort();
    });

    let response = receiver.recv().await;

    runtime.abort();

    ensure!(
        Some("timeout".to_string()) != response,
        "Timed out after {timeout_min} minutes"
    );

    Ok(response.unwrap())
}
