use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use async_compression::tokio::bufread::GzipDecoder;
use async_zip::write::ZipFileWriter;
use async_zip::ZipEntryBuilder;
use ignore::WalkBuilder;
use tokio::fs;
use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
use tokio_tar::Archive;

use super::utils::{get_files_from_volume, send_files_to_volume};
use crate::commands::ignite::types::Deployment;
use crate::commands::volumes::utils::parse_target_from_path_like;
use crate::state::http::HttpClient;
use crate::state::State;

#[derive(Debug)]
/// A file system like object that can be either local or remote
/// This is used to abstract away the differences between local and remote file
/// systems and allow for a single implementation of the copy command
pub enum FsLike<'a> {
    Local(LocalFs),
    Remote(RemoteFs<'a>),
}

impl<'a> FsLike<'a> {
    pub fn new_local(path: &str) -> Self {
        Self::Local(LocalFs { path: path.into() })
    }

    pub fn new_remote(http: &'a HttpClient, deployment: &str, volume: &str, path: &str) -> Self {
        Self::Remote(RemoteFs {
            http,
            deployment: deployment.into(),
            volume: volume.into(),
            path: path.into(),
        })
    }

    pub async fn read(&self) -> Result<(bool, Vec<u8>)> {
        log::debug!("Reading from {}", self.point());

        match self {
            Self::Local(fs) => fs.read().await,
            Self::Remote(fs) => fs.read().await,
        }
    }

    pub async fn write(&self, data: Vec<u8>, packed: bool) -> Result<()> {
        log::debug!("Writing to {}", self.point());

        match self {
            Self::Local(fs) => fs.write(data.as_slice(), packed).await,
            Self::Remote(fs) => fs.write(data, packed).await,
        }
    }

    // Has to take `State` because it needs to get the deployment by name or id
    pub async fn from_str(state: &'a State, s: &str) -> Result<FsLike<'a>> {
        let parsed = parse_target_from_path_like(state, s).await?;

        match parsed {
            (Some((Deployment { id: deployment, .. }, volume)), path) => {
                Ok(Self::new_remote(&state.http, &deployment, &volume, &path))
            }
            (None, path) => Ok(Self::new_local(&path)),
        }
    }

    pub fn is_local(&self) -> bool {
        match self {
            Self::Local(_) => true,
            Self::Remote(_) => false,
        }
    }

    pub fn update_paths(&mut self, path: &str) {
        match self {
            Self::Local(fs) => fs.path = path.into(),
            Self::Remote(fs) => fs.path = path.into(),
        }
    }

    fn path(&self) -> String {
        match self {
            Self::Local(fs) => fs.path.clone(),
            Self::Remote(fs) => fs.path.clone(),
        }
    }

    fn point(&self) -> String {
        match self {
            Self::Local(fs) => format!("local:{}", fs.path),
            Self::Remote(fs) => format!("{}:{}", fs.deployment, fs.path),
        }
    }

    /// Read the contents of the source and write them to the target
    /// Returns the number of bytes written (compressed)
    pub async fn to(self, mut target: Self) -> Result<usize> {
        let (packed, bytes) = self.read().await?;

        // if not packed validate and update paths
        // so it behaves as close to mv/cp as possible
        if !packed {
            let mut path = PathBuf::from(&target.path());

            // check if the target is a directory
            let is_dir = if path.exists() { path.is_dir() } else { false };

            if is_dir {
                let src_path = PathBuf::from(&self.path());

                path = path.join(src_path.file_name().context("No file name")?);
            }

            // update the paths in the target
            target.update_paths(path.to_str().context("Could not get path")?);
        }

        let size = bytes.len();

        log::debug!(
            "Writing {size} bytes to {}, packed: {packed}",
            target.point()
        );

        target.write(bytes, packed).await?;

        Ok(size)
    }
}

#[derive(Debug)]
pub struct LocalFs {
    pub path: String,
}

impl LocalFs {
    async fn read(&self) -> Result<(bool, Vec<u8>)> {
        let path = Path::new(&self.path).canonicalize()?;

        // return early if the path is a file
        if !path.is_dir() {
            return Ok((false, fs::read(&path).await?));
        }

        let mut zip = ZipFileWriter::new(BufWriter::new(vec![]));

        // walk the directory and add files to the zip
        let walker = WalkBuilder::new(&path)
            .standard_filters(false)
            .hidden(false)
            .git_ignore(false)
            .git_exclude(false)
            .ignore(false)
            .build();

        let prefix = path.parent().context("Could not get parent")?;

        for entry in walker {
            match entry {
                Ok(entry) => {
                    // skip directories
                    if entry
                        .file_type()
                        .context("Could not get file type")?
                        .is_dir()
                    {
                        continue;
                    }

                    let relative = entry
                        .path()
                        .strip_prefix(prefix)?
                        .to_string_lossy()
                        .to_string();

                    log::debug!("Adding `{relative}` to zip");

                    let zip_entry =
                        ZipEntryBuilder::new(relative, async_zip::Compression::Deflate).build();

                    let data = fs::read(entry.path()).await?;

                    zip.write_entry_whole(zip_entry, &data).await?;
                }
                Err(why) => log::warn!("Error: {why:?}"),
            }
        }

        let mut buff = zip.close().await?;

        log::debug!("Done writing zip");

        buff.flush().await?;

        Ok((true, buff.into_inner()))
    }

    // Data should be a tarball
    async fn write(&self, data: &[u8], packed: bool) -> Result<()> {
        let path = Path::new(&self.path);

        if !path.exists() {
            fs::create_dir_all(if packed {
                path
            } else {
                path.parent().context("Could not get parent")?
            })
            .await?;
        }

        if packed {
            if !path.is_dir() {
                bail!("Target path is not a directory");
            }

            log::debug!("Unpacking tarball to {}", self.path);

            fs::create_dir_all(&self.path).await?;

            let reader = BufReader::new(data);
            let gunzip = GzipDecoder::new(reader);
            let mut tar = Archive::new(gunzip);

            tar.unpack(&self.path)
                .await
                .context("Could not unpack tarball")?;

            return Ok(());
        }

        log::debug!("Writing single file to {}", self.path);

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .await?;

        file.write_all(data).await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct RemoteFs<'a> {
    pub deployment: String,
    pub volume: String,
    pub path: String,
    pub http: &'a HttpClient,
}

impl<'a> RemoteFs<'a> {
    /// Data should be a zip file
    pub async fn write(&self, data: Vec<u8>, packed: bool) -> Result<()> {
        send_files_to_volume(
            self.http,
            &self.deployment,
            &self.volume,
            &self.path,
            data,
            packed,
        )
        .await
    }

    // Returns a tarball
    pub async fn read(&self) -> Result<(bool, Vec<u8>)> {
        get_files_from_volume(self.http, &self.deployment, &self.volume, &self.path).await
    }
}
