use std::env::temp_dir;
use std::path::PathBuf;

use async_compression::tokio::write::GzipEncoder;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio_tar::Builder as TarBuilder;

use crate::store::hopfile::VALID_HOP_FILENAMES;
use crate::{info, warn};

// default ignore list for tar files
static DEFAULT_IGNORE_LIST: &[&str] = &[
    "/.github",
    ".gitignore",
    ".gitmodules",
    ".DS_Store",
    "/.idea",
    "/.vscode",
];

static VALID_IGNORE_FILENAMES: &[&str] = &[".hopignore", ".gitignore"];

// compress stuff
pub async fn compress(id: String, base_dir: PathBuf) -> Result<String, std::io::Error> {
    let archive_path = temp_dir().join(format!("hop_{}.tar.gz", id));

    // tarball gunzip stuff
    let writer = File::create(archive_path.clone()).await?;
    let writer = GzipEncoder::new(writer);
    let mut archive = TarBuilder::new(writer);
    archive.follow_symlinks(true);

    // .gitignore / .hopignore
    let found_ignore = &find_ignore_files(base_dir.clone()).await;

    info!("Finding files to compress...");
    let files = match found_ignore {
        Some(ignore_path) => gitignore::File::new(ignore_path)
            .unwrap()
            .included_files()
            .unwrap(),
        None => {
            warn!("No ignore file found, creating a .hopignore file");

            // create a new .hopignore file and add some default ignore patterns
            let mut file = File::create(base_dir.join(".hopignore")).await?;
            file.write_all(DEFAULT_IGNORE_LIST.join("\n").as_bytes())
                .await?;
            file.shutdown().await?;

            gitignore::File::new(&base_dir.join(".hopignore").to_path_buf())
                .unwrap()
                .included_files()
                .unwrap()
        }
    };

    // add all found files to the tarball
    for entry in files {
        if VALID_HOP_FILENAMES.contains(&entry.file_name().unwrap().to_str().unwrap()) {
            continue;
        }

        let path = entry.as_path().strip_prefix(&base_dir).unwrap().to_owned();

        archive.append_path_with_name(entry.as_path(), path).await?;
    }

    let mut buff = archive.into_inner().await?;
    buff.shutdown().await?;
    let mut buff = buff.into_inner();
    buff.shutdown().await?;

    Ok(archive_path.to_str().unwrap().into())
}

async fn find_ignore_files(path: PathBuf) -> Option<PathBuf> {
    for filename in VALID_IGNORE_FILENAMES {
        let path = path.clone().join(filename);

        if fs::metadata(&path).await.is_ok() {
            return Some(path);
        }
    }

    None
}
