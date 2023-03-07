use anyhow::{bail, Context, Result};

use super::types::Files;
use crate::state::http::HttpClient;

pub async fn get_files_for_path(
    http: &HttpClient,
    deployment: &str,
    volume: &str,
    path: &str,
) -> Result<Files> {
    let path = path_into_uri_safe(path);

    let files = http
        .request::<Files>(
            "GET",
            &format!("/ignite/deployments/{deployment}/volumes/{volume}/files/{path}"),
            None,
        )
        .await?
        .context("Failed to get files for path")?;

    Ok(files)
}

pub fn path_into_uri_safe(path: &str) -> String {
    path.replace('/', "%2F")
}

pub fn permission_to_string(permission: u64) -> Result<String> {
    let permission = u32::from_str_radix(&permission.to_string(), 8)?;

    let mut perms = String::new();

    // mask out the file type
    match permission & 0o170000 {
        // socket
        0o140000 => perms.push('s'),
        // symlink
        0o120000 => perms.push('l'),
        // file
        0o100000 => perms.push('-'),
        // block device
        0o060000 => perms.push('b'),
        // directory
        0o040000 => perms.push('d'),
        // char device
        0o020000 => perms.push('c'),
        // fifo (named pipe)
        0o010000 => perms.push('p'),

        _ => bail!("Unknown file type"),
    }

    // the file permissions are three octal digits
    for i in 0..3 {
        let shifted = permission >> (6 - (i * 3));

        if shifted & 0o4 != 0 {
            perms.push('r');
        } else {
            perms.push('-');
        }

        if shifted & 0o2 != 0 {
            perms.push('w');
        } else {
            perms.push('-');
        }

        if shifted & 0o1 != 0 {
            perms.push('x');
        } else {
            perms.push('-');
        }
    }

    Ok(perms)
}
