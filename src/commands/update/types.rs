use std::{fmt::Display, num::ParseIntError, str::FromStr};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prelease: Option<u32>,
}

impl FromStr for Version {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tag = if s.starts_with("v") { &s[1..] } else { s };

        let mut parts = tag.split('.');

        let major = parts.next().unwrap_or("0").parse()?;
        let minor = parts.next().unwrap_or("0").parse()?;
        let patch = parts.next().unwrap_or("0").parse()?;
        let prelease = match tag.split("-").nth(1) {
            Some(prelease) => Some(prelease.parse()?),
            None => None,
        };

        Ok(Self {
            major,
            minor,
            patch,
            prelease,
        })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut version = format!("{}.{}.{}", self.major, self.minor, self.patch);

        if let Some(prelease) = &self.prelease {
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
        // || (self.major == other.major
        //     && self.minor == other.minor
        //     && self.patch == other.patch
        //     && self.prelease.is_some()
        //     && other.prelease.is_none())
        // || (self.major == other.major
        //     && self.minor == other.minor
        //     && self.patch == other.patch
        //     && self.prelease.is_some()
        //     && other.prelease.is_some()
        //     && self.prelease.unwrap() > other.prelease.unwrap())
    }
}

#[derive(Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub prerelease: bool,
}
