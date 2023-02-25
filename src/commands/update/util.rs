use std::env::temp_dir;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command as Cmd;

use anyhow::{anyhow, ensure, Context, Result};
use runas::Command as SudoCmd;
use tokio::fs;

use crate::state::http::HttpClient;
use crate::utils::is_writable;

#[cfg(feature = "update")]
pub const HOP_CLI_DOWNLOAD_URL: &str = "https://github.com/hopinc/hop_cli/releases/download";

#[cfg(not(windows))]
const COMPRESSED_FILE_EXTENSION: &str = "tar.gz";

#[cfg(windows)]
const COMPRESSED_FILE_EXTENSION: &str = "zip";

pub async fn download(
    http: &HttpClient,
    base_url: &str,
    version: &str,
    filename: &str,
) -> Result<PathBuf> {
    log::info!("Downloading {filename}@{version}");

    let response = http
        .client
        .get(&format!(
            "{base_url}/{version}/{filename}.{COMPRESSED_FILE_EXTENSION}"
        ))
        .send()
        .await
        .context("Failed to get latest release")?;

    ensure!(
        response.status().is_success(),
        "Failed to get latest release: {}",
        response.status()
    );

    let data = response
        .bytes()
        .await
        .context("Failed to get latest release")?;

    let packed_temp = temp_dir().join(filename);

    log::debug!("Downloading to: {packed_temp:?}");

    fs::write(&packed_temp, &data).await?;

    Ok(packed_temp)
}

#[cfg(not(windows))]
pub async fn unpack(packed_temp: &PathBuf, filename: &str) -> Result<PathBuf> {
    use async_compression::tokio::bufread::GzipDecoder;
    use tokio::io::BufReader;
    use tokio_tar::Archive;

    let file = fs::File::open(packed_temp).await?;
    let reader = BufReader::new(file);
    let gunzip = GzipDecoder::new(reader);
    let mut tar = Archive::new(gunzip);

    let unpack_dir = temp_dir().join(format!("extract-tmp-{filename}"));

    // clean up any existing unpacked files
    fs::remove_dir_all(unpack_dir.clone()).await.ok();
    fs::create_dir_all(unpack_dir.clone()).await?;

    tar.unpack(&unpack_dir).await?;

    let exe = unpack_dir.join(filename);

    log::debug!("Unpacked to: {exe:?}");

    Ok(exe)
}

#[cfg(not(windows))]
pub async fn swap_exe_command(
    non_elevated_args: &mut Vec<OsString>,
    elevated_args: &mut Vec<OsString>,
    old_exe: PathBuf,
    new_exe: PathBuf,
) {
    if is_writable(&old_exe).await {
        non_elevated_args
    } else {
        elevated_args
    }
    .push(format!("mv {} {}", new_exe.display(), old_exe.display()).into());
}

// disable on macos because its doesnt allow to edit completions like this
#[cfg(all(
    not(any(target_os = "windows", target_os = "macos")),
    feature = "update"
))]
pub async fn create_completions_commands(
    non_elevated_args: &mut Vec<OsString>,
    elevated_args: &mut Vec<OsString>,
    exe_path: PathBuf,
) {
    let command = format!(
        "&& mkdir -p /usr/share/zsh/site-functions && {} completions zsh > /usr/share/zsh/site-functions/_hop 2> /dev/null && chmod 644 /usr/share/zsh/site-functions/_hop",
        exe_path.display()
    );

    if is_writable(&PathBuf::from("/usr/share/zsh/site-functions/_hop")).await {
        non_elevated_args.push(command.into());
    } else {
        elevated_args.push(command.into());
    };

    let command = format!(
        "&& mkdir -p /usr/share/fish/completions && {} completions fish > /usr/share/fish/completions/hop.fish 2> /dev/null && chmod 644 /usr/share/fish/completions/hop.fish",
        exe_path.display()
    );

    if is_writable(&PathBuf::from("/usr/share/fish/completions/hop.fish")).await {
        non_elevated_args.push(command.into());
    } else {
        elevated_args.push(command.into());
    };

    let command = format!(
        "&& mkdir -p /usr/share/bash-completion/completions && {} completions bash > /usr/share/bash-completion/completions/hop 2> /dev/null && chmod 644 /usr/share/bash-completion/completions/hop",
        exe_path.display()
    );

    if is_writable(&PathBuf::from("/usr/share/bash-completion/completions/hop")).await {
        non_elevated_args.push(command.into());
    } else {
        elevated_args.push(command.into());
    };
}

#[cfg(windows)]
pub async fn unpack(packed_temp: &PathBuf, filename: &str) -> Result<PathBuf> {
    use std::vec;

    use async_zip::read::stream::ZipFileReader;
    use tokio::io::AsyncReadExt;

    log::debug!("Unpacking: {packed_temp:?}");

    let stream = fs::File::open(packed_temp).await?;
    // seeking breaks the zips since its a single file
    let zip = ZipFileReader::new(stream);

    let exe = temp_dir().join(format!("{filename}.exe"));

    let mut data = vec![];

    // unpack the only file
    zip.next_entry()
        .await?
        .context("brokey entry")?
        .reader()
        .read_to_end(&mut data)
        .await?;

    fs::write(&exe, &data).await?;

    log::debug!("Unpacked to: {exe:?}");

    Ok(exe)
}

#[cfg(windows)]
pub async fn swap_exe_command(
    non_elevated_args: &mut Vec<OsString>,
    elevated_args: &mut Vec<OsString>,
    old_exe: PathBuf,
    new_exe: PathBuf,
) {
    let temp_delete = temp_dir().join(".hop.tmp");

    if is_writable(&old_exe).await {
        non_elevated_args
    } else {
        elevated_args
    }
    .extend(vec![
        "del".into(),
        temp_delete.clone().into(),
        "2> nul".into(),
        "| true".into(),
        "&".into(),
        "move".into(),
        old_exe.clone().into(),
        temp_delete.clone().into(),
        "&".into(),
        "move".into(),
        new_exe.clone().into(),
        old_exe.clone().into(),
        "&".into(),
        "del".into(),
        temp_delete.clone().into(),
        "2> nul".into(),
        "| true".into(),
    ]);
}

// is windows autocomplete even supported?
#[cfg(all(any(target_os = "windows", target_os = "macos"), feature = "update"))]
#[inline]
pub async fn create_completions_commands(
    _non_elevated_args: &mut [OsString],
    _elevated_args: &mut [OsString],
    _exe_path: PathBuf,
) {
}

#[cfg(windows)]
const CMD: &str = "cmd.exe";
#[cfg(not(windows))]
const CMD: &str = "sh";

#[cfg(windows)]
const CMD_ARGS: &[&str] = &["/C"];
#[cfg(not(windows))]
const CMD_ARGS: &[&str] = &["-c"];

pub async fn execute_commands(
    non_elevated_args: &Vec<OsString>,
    elevated_args: &Vec<OsString>,
) -> Result<()> {
    if !elevated_args.is_empty() {
        log::debug!("elevated commands: {elevated_args:?}");

        SudoCmd::new(CMD)
            .args(CMD_ARGS)
            .args(elevated_args)
            .status()?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow!("Failed to execute the command"))?;
    }

    if !non_elevated_args.is_empty() {
        log::debug!("non-elevated commands: {non_elevated_args:?}");

        Cmd::new(CMD)
            .args(CMD_ARGS)
            .args(non_elevated_args)
            .status()?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow!("Failed to execute the command"))?;
    }

    Ok(())
}
