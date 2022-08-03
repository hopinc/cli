use std::fmt::Display;
use std::num::ParseIntError;

use serde::Deserialize;

use super::parse;

#[derive(Debug, Deserialize, Clone)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub release: Option<u16>,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut version = format!("{}.{}.{}", self.major, self.minor, self.patch);

        if let Some(prelease) = &self.release {
            version += format!("-{}", prelease).as_str();
        }

        write!(f, "{}", version)
    }
}

impl Version {
    pub fn is_newer(&self, other: &Version) -> bool {
        self.major > other.major
            || (self.major == other.major && self.minor > other.minor)
            || (self.major == other.major && self.minor == other.minor && self.patch > other.patch)
            || (self.major == other.major
                && self.minor == other.minor
                && self.patch == other.patch
                && self.release.is_some()
                && other.release.is_none())
            || (self.major == other.major
                && self.minor == other.minor
                && self.patch == other.patch
                && self.release.is_some()
                && other.release.is_some()
                && self.release.unwrap() > other.release.unwrap())
    }

    pub fn from_string(s: &str) -> Result<Self, ParseIntError> {
        let (major, minor, patch, prelease) = parse::version(s)?;

        Ok(Self {
            major,
            minor,
            patch,
            release: prelease,
        })
    }
}

#[derive(Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub prerelease: bool,
    pub draft: bool,
}
