use anyhow::{bail, Context, Result};
use chrono::Datelike;

use super::types::{File, Files};
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

/// Convert a path into a URI safe(ish) string
pub fn path_into_uri_safe(path: &str) -> String {
    path.replace('/', "%2F")
}

/// Convert a permission number into a string
/// 40755 -> drwxr-xr-x
/// 100644 -> -rw-r--r--
fn permission_to_string(permission: u64) -> Result<String> {
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

pub fn format_file(file: &File) -> Result<String> {
    let date =
        chrono::DateTime::parse_from_rfc3339(&file.updated_at).context("Failed to parse date")?;

    let date = if date.year() == chrono::Local::now().year() {
        date.format("%b %d %H:%M")
    } else {
        date.format("%b %d %Y")
    };

    let res = format!(
        "{}\t{}\t{}\t{}",
        permission_to_string(file.permissions)?,
        file.size,
        date,
        file.name
    );

    Ok(res)
}

pub fn get_volume_from_deployment(deployment: &str) -> Result<String> {
    let tail = deployment
        .split('_')
        .nth(1)
        .context("Failed to get volume from deployment")?;

    return Ok(format!("volume_{tail}"));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_path_into_uri_safe() {
        assert_eq!(path_into_uri_safe("/"), "%2F");
        assert_eq!(path_into_uri_safe("/home"), "%2Fhome");
        assert_eq!(path_into_uri_safe("/home/"), "%2Fhome%2F");
        assert_eq!(path_into_uri_safe("/home/user"), "%2Fhome%2Fuser");
        assert_eq!(path_into_uri_safe("/home/user/"), "%2Fhome%2Fuser%2F");
        assert_eq!(
            path_into_uri_safe("/home/user/file"),
            "%2Fhome%2Fuser%2Ffile"
        );
    }

    #[test]
    fn test_permission_to_string() {
        assert_eq!(permission_to_string(40755).unwrap(), "drwxr-xr-x");
        assert_eq!(permission_to_string(100644).unwrap(), "-rw-r--r--");
        assert_eq!(permission_to_string(100777).unwrap(), "-rwxrwxrwx");
    }
}
