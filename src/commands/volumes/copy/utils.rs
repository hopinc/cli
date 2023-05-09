use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use reqwest::multipart::{Form, Part};
use serde_json::Value;

use crate::commands::volumes::utils::path_into_uri_safe;
use crate::state::http::HttpClient;

pub async fn send_files_to_volume<'l>(
    http: &HttpClient,
    deployment: &str,
    volume: &str,
    path: &str,
    data: Vec<u8>,
    packed: bool,
) -> Result<()> {
    let url = format!("/ignite/deployments/{deployment}/volumes/{volume}/files",);

    let (path, filename) = if packed {
        (path, "archive.zip".to_string())
    } else {
        let buf = PathBuf::from(path);

        (
            path,
            buf.file_name()
                .context("No file name")?
                .to_str()
                .context("Invalid file name")?
                .to_string(),
        )
    };

    let form = Form::new()
        .part("file", Part::bytes(data).file_name(filename))
        .part("path", Part::text(path.to_string()));

    log::debug!("Packed: {}", packed);

    let response = http
        .client
        .post(format!("{}{url}", http.base_url))
        .header("X-No-Unpacking", (!packed).to_string())
        .multipart(form)
        .send()
        .await?;

    // since it should be a 204, we don't need to parse the response data
    http.handle_response::<Value>(response).await?;

    Ok(())
}

pub async fn get_files_from_volume(
    http: &HttpClient,
    deployment: &str,
    volume: &str,
    path: &str,
) -> Result<(bool, Vec<u8>)> {
    let path = path_into_uri_safe(path);

    let url = format!("/ignite/deployments/{deployment}/volumes/{volume}/files/{path}");

    let response = http
        .client
        .get(format!("{}{url}", http.base_url))
        .query(&[("stream", "true")])
        .send()
        .await?;

    log::debug!("Response headers: {:#?}", response.headers());

    // check header for content type
    let packed = response
        .headers()
        .get("x-directory")
        .context("No content type header")?
        .to_str()?
        .to_lowercase()
        == "true";

    let data = match response.status() {
        reqwest::StatusCode::OK => response.bytes().await?,
        reqwest::StatusCode::NOT_FOUND => bail!("File not found"),
        status => {
            // bogus type
            http.handle_error::<Vec<u8>>(response, status).await?;

            unreachable!("handle_error should have returned an error");
        }
    };

    Ok((packed, data.to_vec()))
}
