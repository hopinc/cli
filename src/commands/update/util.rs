use std::env::temp_dir;
use std::path::PathBuf;
use std::process::Command as Cmd;

use runas::Command as SudoCmd;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

use super::types::{GithubRelease, Version};
use crate::config::{ARCH, VERSION};
use crate::state::http::HttpClient;

const RELEASE_URL: &str = "https://api.github.com/repos/hopinc/hop_cli/releases";

pub async fn check_version(beta: bool, silent: bool) -> (bool, String) {
    let http = HttpClient::new(None, None);

    let response = match http.client.get(RELEASE_URL).send().await {
        Ok(response) => response,
        Err(e) => {
            if !silent {
                log::error!("Failed to check for updates: {}", e);
            }

            return (false, VERSION.to_string());
        }
    };

    if !response.status().is_success() {
        log::debug!(
            "Failed to get latest release from Github: {}",
            response.status()
        );
        // silently fail if we can't get the latest release
        return (false, "".to_string());
    }

    let data = response
        .json::<Vec<GithubRelease>>()
        .await
        .expect("Failed to parse latest release");

    let latest = if beta {
        // the latest release that can be prereleased
        data
            .iter()
            // skip drafts
            .find(|r| !r.draft)
            .map(|r| r.tag_name.clone())
            .expect("No release found")
    } else {
        // the latest release that is not prereleased
        data
            .iter()
            // skip drafts and prereleases
            .find(|r| !r.prerelease && !r.draft)
            .map(|r| r.tag_name.clone())
            .expect("No beta release found")
    };

    let latest = Version::from_string(&latest).unwrap();
    let current = Version::from_string(VERSION).unwrap();

    if latest.is_newer(&current) {
        (true, latest.to_string())
    } else {
        (false, String::new())
    }
}

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[cfg(not(windows))]
const COMPRESSED_FILE_EXTENSION: &str = "tar.gz";

#[cfg(windows)]
const COMPRESSED_FILE_EXTENSION: &str = "zip";

pub async fn download(http: HttpClient, version: String) -> Result<PathBuf, std::io::Error> {
    let filename = format!(
        "hop-{}-{}.{}",
        ARCH,
        capitalize(&sys_info::os_type().unwrap_or_else(|_| "Unknown".to_string())),
        COMPRESSED_FILE_EXTENSION
    );

    log::info!("Downloading {}@{}", filename, version);

    let response = http
        .client
        .get(&format!(
            "https://github.com/hopinc/hop_cli/releases/download/v{}/{}",
            version, filename
        ))
        .send()
        .await
        .expect("Failed to get latest release");

    assert!(
        response.status().is_success(),
        "Failed to get latest release: {}",
        response.status()
    );

    let data = response
        .bytes()
        .await
        .expect("Failed to get latest release");

    let packed_temp = temp_dir().join(filename);

    log::debug!("Downloading to: {}", packed_temp.display());

    let mut file = File::create(&packed_temp).await?;

    file.write_all(&data).await?;

    Ok(packed_temp)
}

#[cfg(not(windows))]
pub async fn unpack(packed_temp: PathBuf) -> Result<PathBuf, std::io::Error> {
    use async_compression::tokio::bufread::GzipDecoder;
    use tokio::io::BufReader;
    use tokio_tar::Archive;

    let file = File::open(packed_temp).await?;
    let reader = BufReader::new(file);
    let gunzip = GzipDecoder::new(reader);
    let mut tar = Archive::new(gunzip);

    let unpack_dir = temp_dir().join("hop-extract");

    // clean up any existing unpacked files
    fs::remove_dir_all(unpack_dir.clone()).await.ok();
    fs::create_dir_all(unpack_dir.clone()).await?;

    tar.unpack(&unpack_dir).await?;

    let exe = unpack_dir.join("hop");

    log::debug!("Unpacked to: {}", exe.display());

    Ok(exe)
}

#[cfg(not(windows))]
pub async fn swap_executables(old_exe: PathBuf, new_exe: PathBuf) -> Result<(), std::io::Error> {
    let elevate = !is_writable(&old_exe).await;

    if elevate {
        SudoCmd::new("mv").arg(&new_exe).arg(&old_exe).status()?;
    } else {
        Cmd::new("mv").arg(&new_exe).arg(&old_exe).status()?;
    }

    Ok(())
}

#[cfg(windows)]
pub async fn unpack(packed_temp: PathBuf) -> Result<PathBuf, std::io::Error> {
    use async_zip::read::fs::ZipFileReader;

    let zip = ZipFileReader::new(packed_temp).await.unwrap();

    let exe = temp_dir().join("hop.exe");

    // unpack the only file
    let data = zip
        .entry_reader(0)
        .await
        .expect("brokey entry")
        .read_to_end_crc()
        .await
        .expect("failed to read entry");

    let mut file = File::create(&exe).await?;
    file.write_all(&data).await?;

    log::debug!("Unpacked to: {}", exe.display());

    Ok(exe)
}

#[cfg(windows)]
pub async fn swap_executables(old_exe: PathBuf, new_exe: PathBuf) -> Result<(), std::io::Error> {
    let elevate = !is_writable(&old_exe).await;

    let temp_delete = temp_dir().join(".hop.tmp");

    if elevate {
        SudoCmd::new("cmd")
            .arg("/c")
            .arg("move")
            .arg(&old_exe)
            .arg(&temp_delete)
            .arg("&")
            .arg("move")
            .arg(&new_exe)
            .arg(&old_exe)
            .arg("&")
            .arg("del")
            .arg(&temp_delete)
            .status()?;
    } else {
        Cmd::new("cmd")
            .arg("/c")
            .arg("move")
            .arg(&old_exe)
            .arg(&temp_delete)
            .arg("&")
            .arg("move")
            .arg(&new_exe)
            .arg(&old_exe)
            .arg("&")
            .arg("del")
            .arg(&temp_delete)
            .status()?;
    }

    Ok(())
}

async fn is_writable(path: &PathBuf) -> bool {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .is_ok()
}
