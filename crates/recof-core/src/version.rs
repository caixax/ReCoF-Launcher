//! Version numbers, compared the way semver says.
//!
//! Three places depend on getting this right: deciding whether a release is
//! newer than what is installed, deciding whether the installed ReCoF
//! demands a newer launcher, and ordering the version store. A dedicated
//! crate would do, but the rules are short, the tests are the real value,
//! and every dependency is bytes in the executable.

use std::cmp::Ordering;
use std::fmt;

/// A release version: `major.minor.patch` with an optional prerelease.
///
/// Build metadata after a `+` is parsed and kept for display but, as semver
/// requires, ignored when comparing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    /// Dot separated identifiers after a `-`, empty for a final release.
    pub prerelease: Vec<String>,
    /// Everything after a `+`, ignored in comparisons.
    pub build: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VersionError {
    #[error("{0:?} is not a version number")]
    Malformed(String),
}

impl Version {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Version {
            major,
            minor,
            patch,
            prerelease: Vec::new(),
            build: None,
        }
    }

    /// Parses `1.2.3`, `v1.2.3`, `1.2.3-beta.1` or `1.2.3+build7`.
    ///
    /// A leading `v` is accepted because that is how git tags and GitHub
    /// release names are written.
    pub fn parse(text: &str) -> Result<Self, VersionError> {
        let malformed = || VersionError::Malformed(text.to_owned());

        let trimmed = text.trim();
        let without_v = trimmed
            .strip_prefix('v')
            .or_else(|| trimmed.strip_prefix('V'))
            .unwrap_or(trimmed);

        let (rest, build) = match without_v.split_once('+') {
            Some((rest, build)) if !build.is_empty() => (rest, Some(build.to_owned())),
            Some(_) => return Err(malformed()),
            None => (without_v, None),
        };

        let (core, prerelease) = match rest.split_once('-') {
            Some((core, pre)) if !pre.is_empty() => (core, pre),
            Some(_) => return Err(malformed()),
            None => (rest, ""),
        };

        let mut numbers = core.split('.');
        let major = parse_number(numbers.next().ok_or_else(malformed)?).ok_or_else(malformed)?;
        let minor = parse_number(numbers.next().ok_or_else(malformed)?).ok_or_else(malformed)?;
        let patch = parse_number(numbers.next().ok_or_else(malformed)?).ok_or_else(malformed)?;
        if numbers.next().is_some() {
            return Err(malformed());
        }

        let prerelease: Vec<String> = if prerelease.is_empty() {
            Vec::new()
        } else {
            prerelease.split('.').map(str::to_owned).collect()
        };
        if prerelease.iter().any(|part| part.is_empty()) {
            return Err(malformed());
        }

        Ok(Version {
            major,
            minor,
            patch,
            prerelease,
            build,
        })
    }

    /// Whether this is a prerelease, which is what the beta channel ships.
    pub fn is_prerelease(&self) -> bool {
        !self.prerelease.is_empty()
    }
}

/// Rejects leading zeroes and anything that is not a plain number, so
/// `01.2.3` and `1.2.x` are refused rather than quietly misread.
fn parse_number(text: &str) -> Option<u64> {
    if text.is_empty() || (text.len() > 1 && text.starts_with('0')) {
        return None;
    }
    if !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
            .then_with(|| compare_prerelease(&self.prerelease, &other.prerelease))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Semver rule: a version with a prerelease is older than the same version
/// without one, and prerelease identifiers compare field by field, numeric
/// ones numerically and below alphanumeric ones.
fn compare_prerelease(left: &[String], right: &[String]) -> Ordering {
    match (left.is_empty(), right.is_empty()) {
        (true, true) => return Ordering::Equal,
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        (false, false) => {}
    }

    for (a, b) in left.iter().zip(right.iter()) {
        let ordering = match (parse_number(a), parse_number(b)) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => a.cmp(b),
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    left.len().cmp(&right.len())
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.prerelease.is_empty() {
            write!(formatter, "-{}", self.prerelease.join("."))?;
        }
        if let Some(build) = &self.build {
            write!(formatter, "+{build}")?;
        }
        Ok(())
    }
}

impl std::str::FromStr for Version {
    type Err = VersionError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Version::parse(text)
    }
}

impl serde::Serialize for Version {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Version {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Version::parse(&text).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).unwrap_or_else(|error| panic!("{text:?}: {error}"))
    }

    #[test]
    fn parses_a_plain_version() {
        let version = v("1.2.3");
        assert_eq!((version.major, version.minor, version.patch), (1, 2, 3));
        assert!(version.prerelease.is_empty());
        assert_eq!(version.build, None);
    }

    #[test]
    fn accepts_the_v_prefix_git_tags_use() {
        assert_eq!(v("v0.4.1"), Version::new(0, 4, 1));
        assert_eq!(v("V0.4.1"), Version::new(0, 4, 1));
        assert_eq!(v("  v0.4.1  "), Version::new(0, 4, 1));
    }

    #[test]
    fn parses_a_prerelease_and_build() {
        let version = v("1.0.0-beta.2+20261026");
        assert_eq!(version.prerelease, vec!["beta", "2"]);
        assert_eq!(version.build.as_deref(), Some("20261026"));
        assert!(version.is_prerelease());
    }

    #[test]
    fn rejects_what_is_not_a_version() {
        for bad in [
            "",
            "1",
            "1.2",
            "1.2.3.4",
            "1.2.x",
            "01.2.3",
            "1.02.3",
            "-1.2.3",
            "1.2.3-",
            "1.2.3+",
            "1.2.3-beta..1",
            "a.b.c",
            "1.2.-3",
        ] {
            assert!(Version::parse(bad).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn orders_by_each_number_in_turn() {
        assert!(v("1.0.0") < v("2.0.0"));
        assert!(v("1.9.0") < v("1.10.0"));
        assert!(v("1.0.9") < v("1.0.10"));
        assert_eq!(v("1.2.3"), v("1.2.3"));
    }

    #[test]
    fn a_prerelease_is_older_than_the_release() {
        assert!(v("1.0.0-beta") < v("1.0.0"));
        assert!(v("1.0.0-rc.1") < v("1.0.0"));
        assert!(v("0.9.9") < v("1.0.0-alpha"));
    }

    #[test]
    fn prerelease_identifiers_follow_semver_precedence() {
        // The example from the semver specification.
        let ordered = [
            "1.0.0-alpha",
            "1.0.0-alpha.1",
            "1.0.0-alpha.beta",
            "1.0.0-beta",
            "1.0.0-beta.2",
            "1.0.0-beta.11",
            "1.0.0-rc.1",
            "1.0.0",
        ];
        for pair in ordered.windows(2) {
            assert!(v(pair[0]) < v(pair[1]), "{} < {}", pair[0], pair[1]);
        }
    }

    #[test]
    fn build_metadata_does_not_change_the_order() {
        assert_eq!(v("1.0.0+a").cmp(&v("1.0.0+b")), Ordering::Equal);
        assert_eq!(v("1.0.0").cmp(&v("1.0.0+build")), Ordering::Equal);
    }

    #[test]
    fn sorting_a_release_list_puts_the_newest_last() {
        let mut list = [v("1.0.0"), v("0.9.0"), v("1.0.0-beta.1"), v("1.1.0")];
        list.sort();
        let shown: Vec<String> = list.iter().map(ToString::to_string).collect();
        assert_eq!(shown, ["0.9.0", "1.0.0-beta.1", "1.0.0", "1.1.0"]);
    }

    #[test]
    fn display_round_trips_through_parse() {
        for text in ["1.2.3", "0.0.1-rc.1", "2.0.0-beta.1+sha.abcdef"] {
            assert_eq!(v(text).to_string(), text);
        }
    }

    #[test]
    fn serde_uses_the_string_form() {
        let version = v("1.2.3-beta.1");
        let json = serde_json::to_string(&version).unwrap();
        assert_eq!(json, "\"1.2.3-beta.1\"");
        assert_eq!(serde_json::from_str::<Version>(&json).unwrap(), version);
        assert!(serde_json::from_str::<Version>("\"not a version\"").is_err());
    }

    #[test]
    fn the_launchers_own_version_parses() {
        // A broken Cargo version would otherwise only show up at runtime.
        assert!(Version::parse(crate::consts::LAUNCHER_VERSION).is_ok());
    }
}
