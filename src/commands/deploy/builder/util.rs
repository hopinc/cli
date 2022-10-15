use std::env::temp_dir;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use async_compression::tokio::write::GzipEncoder;
use hyper::Method;
use ignore::WalkBuilder;
use reqwest::multipart::{Form, Part};
use serde_json::Value;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio_tar::Builder as TarBuilder;

use super::types::{Build, SingleBuild};
use crate::commands::deploy::HOP_BUILD_BASE_URL;
use crate::state::http::HttpClient;
use crate::store::hopfile::VALID_HOP_FILENAMES;

pub async fn builder_post(http: &HttpClient, deployment_id: &str, bytes: Vec<u8>) -> Result<Build> {
    let multipart = Form::new().part(
        "file",
        Part::bytes(bytes)
            .file_name("deployment.tar.gz")
            .mime_str("application/x-gzip")?,
    );

    let response = http
        .client
        .request(
            Method::POST,
            format!("{HOP_BUILD_BASE_URL}/deployments/{deployment_id}/builds",).as_str(),
        )
        .header("content_type", "multipart/form-data".to_string())
        .multipart(multipart)
        .send()
        .await?;

    let build = http
        .handle_response::<SingleBuild>(response)
        .await?
        .ok_or_else(|| anyhow!("Could not create build"))?
        .build;

    Ok(build)
}

pub async fn cancel_build(http: &HttpClient, build_id: &str) -> Result<()> {
    http.request::<Value>("POST", &format!("/ignite/builds/{build_id}/cancel"), None)
        .await?;
    Ok(())
}

// default ignore list for tar files
static DEFAULT_IGNORE_LIST: &[&str] = &[
    ".git",
    ".github",
    ".gitmodules",
    ".DS_Store",
    ".idea",
    ".vscode",
];

// compress stuff
pub async fn compress(id: &str, base_dir: PathBuf) -> Result<String> {
    let base_folder_name = Path::new(&id);
    let archive_path = temp_dir().join(format!("hop_{}.tar.gz", id));

    // tarball gunzip stuff
    let writer = File::create(archive_path.clone()).await?;
    let writer = GzipEncoder::new(writer);
    let mut archive = TarBuilder::new(writer);
    archive.follow_symlinks(true);

    log::info!("Finding files to compress...");

    let mut walker = WalkBuilder::new(&base_dir.clone());
    walker.add_ignore(create_global_ignore_file().await);
    walker.add_custom_ignore_filename(".hopignore");
    walker.hidden(false).follow_links(true);

    let walker = walker.build();

    // add all found files to the tarball
    for entry in walker {
        match entry {
            Ok(entry) => {
                log::debug!("Adding {} to tarball", entry.path().display());

                if VALID_HOP_FILENAMES.contains(&entry.file_name().to_str().unwrap()) {
                    continue;
                }

                let path = entry.path().strip_prefix(&base_dir).unwrap().to_owned();

                archive
                    .append_path_with_name(entry.path(), &(*base_folder_name).join(&path))
                    .await?;
            }
            Err(err) => {
                log::warn!("Error walking: {}", err);
            }
        }
    }

    let mut buff = archive.into_inner().await?;
    buff.shutdown().await?;
    let mut buff = buff.into_inner();
    buff.shutdown().await?;

    Ok(archive_path.to_str().unwrap().into())
}

async fn create_global_ignore_file() -> PathBuf {
    let path = temp_dir().join(".hopignore");
    let mut file = File::create(path.clone())
        .await
        .expect("Could not create global ignore file");

    file.write_all(DEFAULT_IGNORE_LIST.join("\n").as_bytes())
        .await
        .expect("Could not write to global ignore file");

    file.shutdown()
        .await
        .expect("Could not close global ignore file");

    path
}
