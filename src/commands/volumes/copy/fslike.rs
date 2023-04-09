use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use async_compression::tokio::bufread::GzipDecoder;
use async_zip::write::ZipFileWriter;
use async_zip::ZipEntryBuilder;
use ignore::WalkBuilder;
use tokio::fs;
use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
use tokio_tar::Archive;

use super::utils::{get_files_from_volume, send_zip_to_volume};
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
        log::debug!("Reading from {}", self.target());

        match self {
            Self::Local(fs) => Ok((true, fs.read().await?)),
            Self::Remote(fs) => fs.read().await,
        }
    }

    pub async fn write(&self, data: &[u8], packed: bool) -> Result<()> {
        log::debug!("Writing to {}", self.target());

        match self {
            Self::Local(fs) => fs.write(data, packed).await,
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

    /// Read the contents of the source and write them to the target
    /// Returns the number of bytes written (compressed)
    pub async fn to(self, mut target: Self) -> Result<usize> {
        let (packed, bytes) = self.read().await?;

        // if not packed validate and update paths
        // so it behaves as close to mv/cp as possible
        if !packed {
            let mut path = PathBuf::from(&target.target());

            // check if the target is a directory
            let is_dir = if path.exists() { path.is_dir() } else { false };

            if is_dir {
                let src_path = PathBuf::from(&self.target());

                path = path.join(src_path.file_name().context("No file name")?);
            }

            // update the paths in the target
            target.update_paths(path.to_str().context("Could not get path")?);
        }

        let size = bytes.len();

        target.write(&bytes, packed).await?;

        Ok(size)
    }

    fn target(&self) -> String {
        match self {
            Self::Local(fs) => fs.path.clone(),
            Self::Remote(fs) => format!("{}:{}", fs.deployment, fs.path),
        }
    }
}

#[derive(Debug)]
pub struct LocalFs {
    pub path: String,
}

impl LocalFs {
    async fn read(&self) -> Result<Vec<u8>> {
        let buff = BufWriter::new(vec![]);
        let mut zip = ZipFileWriter::new(buff);

        let path = Path::new(&self.path).canonicalize()?;

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

        Ok(buff.into_inner())
    }

    // Data should be a tarball
    async fn write(&self, data: &[u8], packed: bool) -> Result<()> {
        let path = Path::new(&self.path);

        if !path.exists() {
            fs::create_dir_all(&path.parent().context("Could not get parent")?).await?;
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
    pub async fn write(&self, data: &[u8], packed: bool) -> Result<()> {
        send_zip_to_volume(
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
