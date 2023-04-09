use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use reqwest::multipart::{Form, Part};
use serde_json::Value;

use crate::{commands::volumes::utils::path_into_uri_safe, state::http::HttpClient};

pub async fn send_zip_to_volume(
    http: &HttpClient,
    deployment: &str,
    volume: &str,
    path: &str,
    zip: &[u8],
    packed: bool,
) -> Result<()> {
    let url = format!("/ignite/deployments/{deployment}/volumes/{volume}/files",);

    let form = Form::new()
        .part(
            "file",
            Part::bytes(zip.to_vec()).file_name(if packed {
                "packed.zip".to_string()
            } else {
                let buf = PathBuf::from(path);

                buf.file_name()
                    .context("No file name")?
                    .to_str()
                    .context("Invalid file name")?
                    .to_string()
            }),
        )
        .part("path", Part::text(path.to_string()));

    let response = http
        .client
        .post(format!("{}{url}", http.base_url))
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

    // check header for content type
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .context("No content type header")?
        .to_str()?;

    let packed = content_type == "application/gzip";

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
