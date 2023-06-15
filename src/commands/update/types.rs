use std::fmt::{Display, Write};
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
            write!(version, "-{prelease}")?;
        }

        write!(f, "{version}")
    }
}

impl Version {
    pub fn is_newer_than(&self, other: &Version) -> bool {
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

mod test {
    #[test]
    fn version_from_string() {
        let version = super::Version::from_string("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.release, None);
    }

    #[test]
    fn version_from_string_with_release() {
        let version = super::Version::from_string("1.2.3-4").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.release, Some(4));
    }

    #[test]
    fn version_is_newer() {
        let version = super::Version::from_string("1.2.4").unwrap();
        let older = super::Version::from_string("1.2.3").unwrap();
        assert!(version.is_newer_than(&older));

        let version = super::Version::from_string("1.2.4").unwrap();
        let older = super::Version::from_string("1.2.3-1").unwrap();
        assert!(version.is_newer_than(&older));

        let version = super::Version::from_string("1.2.3-1").unwrap();
        let older = super::Version::from_string("1.2.3").unwrap();
        assert!(version.is_newer_than(&older));

        let version = super::Version::from_string("1.3.3").unwrap();
        let older = super::Version::from_string("1.2.3").unwrap();
        assert!(version.is_newer_than(&older));

        let version = super::Version::from_string("2.2.3").unwrap();
        let older = super::Version::from_string("1.2.3").unwrap();
        assert!(version.is_newer_than(&older));
    }

    #[test]
    fn version_is_not_newer() {
        let version = super::Version::from_string("1.2.3").unwrap();
        let older = super::Version::from_string("1.2.3").unwrap();
        assert!(!version.is_newer_than(&older));

        let version = super::Version::from_string("1.2.3-1").unwrap();
        let older = super::Version::from_string("1.2.3-1").unwrap();
        assert!(!version.is_newer_than(&older));

        let version = super::Version::from_string("1.2.3").unwrap();
        let older = super::Version::from_string("1.2.3-1").unwrap();
        assert!(!version.is_newer_than(&older));
    }
}

#[derive(Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub prerelease: bool,
    pub draft: bool,
}
