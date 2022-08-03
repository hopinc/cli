use std::num::ParseIntError;

pub fn version(version: &str) -> Result<(u16, u16, u16, Option<u16>), ParseIntError> {
    let tag = if let Some(stripped) = version.strip_prefix('v') {
        stripped
    } else {
        version
    };

    let mut pre = tag.split('-');
    let mut parts = pre.next().unwrap_or(tag).split('.');

    let major = parts.next().unwrap_or("0").parse()?;
    let minor = parts.next().unwrap_or("0").parse()?;
    let patch = parts.next().unwrap_or("0").parse()?;

    let prelease = match pre.next() {
        Some(prelease) => Some(prelease.parse()?),
        None => None,
    };

    Ok((major, minor, patch, prelease))
}
