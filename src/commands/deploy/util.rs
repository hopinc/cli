use std::collections::HashMap;
use std::env::temp_dir;
use std::path::{Path, PathBuf};

use async_compression::tokio::write::GzipEncoder;
use ignore::WalkBuilder;
use regex::Regex;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio_tar::Builder as TarBuilder;

use crate::commands::ignite::util::parse_key_val;
use crate::store::hopfile::VALID_HOP_FILENAMES;

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
pub async fn compress(id: String, base_dir: PathBuf) -> Result<String, std::io::Error> {
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

pub fn validate_deployment_name(name: &str) -> bool {
    let regex = Regex::new(r"^[a-zA-Z0-9-]*$").unwrap();

    regex.is_match(name)
}

pub async fn env_file_to_map(path: PathBuf) -> HashMap<String, String> {
    let mut env = HashMap::new();

    assert!(
        path.exists(),
        "Could not find .env file at {}",
        path.display()
    );

    let file = fs::read_to_string(path).await.unwrap();
    let lines = file.lines();

    for line in lines {
        // ignore comments
        if line.starts_with('#') {
            continue;
        }

        match parse_key_val(line) {
            Ok((key, value)) => {
                env.insert(key, value);
            }
            Err(e) => log::warn!("Failed to parse env file line: {}", e),
        }
    }

    env
}
