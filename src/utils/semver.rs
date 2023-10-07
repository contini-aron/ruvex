use anyhow::{anyhow, Result};
use core::fmt::Display;
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::min;
use std::cmp::Ordering;
use std::str::FromStr;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SemVer {
    pub major: u128,
    pub minor: u128,
    pub patch: u128,
    pub pre_release: Option<String>,
    pub build_meta: Option<String>,
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq)]
pub enum SemVerChangeType {
    None,
    Patch,
    Minor,
    Major,
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn cmp_prerelease(pre_a: Option<String>, pre_b: Option<String>) -> Ordering {
    if let Some(pre_a_text) = pre_a {
        match pre_b {
            None => Ordering::Less,
            Some(pre_b_text) => {
                let ids_a = pre_a_text.split('.').collect::<Vec<&str>>();
                let ids_b = pre_b_text.split('.').collect::<Vec<&str>>();
                let min_len = min(ids_a.len(), ids_b.len());

                for i in 0..min_len {
                    let num_a = ids_a[i].parse::<u128>();
                    let num_b = ids_b[i].parse::<u128>();

                    if let (Ok(a), Ok(b)) = (num_a, num_b) {
                        match a.cmp(&b) {
                            Ordering::Equal => continue,
                            Ordering::Greater => return Ordering::Greater,
                            Ordering::Less => return Ordering::Less,
                        }
                    } else {
                        match (*ids_a[i]).cmp(ids_b[i]) {
                            Ordering::Equal => continue,
                            Ordering::Greater => return Ordering::Greater,
                            Ordering::Less => return Ordering::Less,
                        }
                    }
                }
                ids_a.len().cmp(&ids_b.len())
            }
        }
    } else {
        match pre_b {
            None => Ordering::Equal,
            Some(_text) => Ordering::Greater,
        }
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        // compare major
        match self.major.cmp(&other.major) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            // compare minor
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Greater => Ordering::Greater,
                Ordering::Less => Ordering::Less,
                Ordering::Equal => match self.patch.cmp(&other.patch) {
                    // compare patch
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Less => Ordering::Less,
                    Ordering::Equal => {
                        cmp_prerelease(self.pre_release.clone(), other.pre_release.clone())
                    }
                },
            },
        }
    }
}

impl SemVer {
    pub fn new(
        major: u128,
        minor: u128,
        patch: u128,
        pre_release: Option<String>,
        build_meta: Option<String>,
    ) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release,
            build_meta,
        }
    }
}

impl Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.pre_release, &self.build_meta) {
            (Some(pre_release), Some(build_meta)) => f.write_fmt(format_args!(
                "{}.{}.{}-{}+{}",
                self.major, self.minor, self.patch, pre_release, build_meta
            )),
            (Some(pre_release), None) => f.write_fmt(format_args!(
                "{}.{}.{}-{}",
                self.major, self.minor, self.patch, pre_release
            )),
            (None, Some(build_meta)) => f.write_fmt(format_args!(
                "{}.{}.{}+{}",
                self.major, self.minor, self.patch, build_meta
            )),
            (None, None) => {
                f.write_fmt(format_args!("{}.{}.{}", self.major, self.minor, self.patch))
            }
        }
    }
}

impl FromStr for SemVer {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let error = s.to_owned();

        lazy_static! {
            // See https://semver.org/spec/v2.0.0.html#is-there-a-suggested-regular-expression-regex-to-check-a-semver-string
            static ref RE: Regex = Regex::new(r"^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$").unwrap();
        }

        let Some(captures) = RE.captures(s) else {
            return Err(anyhow!(error));
        };

        let Some(major) = captures.name("major") else {
            return Err(anyhow!(error));
        };
        let Some(minor) = captures.name("minor") else {
            return Err(anyhow!(error));
        };
        let Some(patch) = captures.name("patch") else {
            return Err(anyhow!(error));
        };

        let Ok(major) = major.as_str().parse() else {
            return Err(anyhow!(error));
        };
        let Ok(minor) = minor.as_str().parse() else {
            return Err(anyhow!(error));
        };
        let Ok(patch) = patch.as_str().parse() else {
            return Err(anyhow!(error));
        };

        let pre_release = captures.name("prerelease").map(|m| m.as_str().to_owned());
        let build_meta = captures
            .name("buildmetadata")
            .map(|m| m.as_str().to_owned());

        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
            build_meta,
        })
    }
}

#[cfg(test)]
mod test {

    use crate::utils::semver::SemVer;
    const PRECEDENCE_STRINGS: [&str; 24] = [
        "0.1.0-alpha",
        "0.1.0-alpha.1",
        "0.1.0-alpha.beta",
        "0.1.0-beta",
        "0.1.0-beta.2",
        "0.1.0-beta.11",
        "0.1.0-rc.1",
        "0.1.0",
        "0.1.1-alpha",
        "0.1.1-alpha.1",
        "0.1.1-alpha.beta",
        "0.1.1-beta",
        "0.1.1-beta.2",
        "0.1.1-beta.11",
        "0.1.1-rc.1",
        "0.1.1",
        "1.0.0-alpha",
        "1.0.0-alpha.1",
        "1.0.0-alpha.beta",
        "1.0.0-beta",
        "1.0.0-beta.2",
        "1.0.0-beta.11",
        "1.0.0-rc.1",
        "1.0.0",
    ];
    #[test]
    fn max() {
        assert_eq!(
            "1.0.0".parse::<SemVer>().unwrap(),
            PRECEDENCE_STRINGS
                .into_iter()
                .map(|x| x.parse::<SemVer>().unwrap())
                .max()
                .unwrap()
        );
    }
    #[test]
    fn min() {
        assert_eq!(
            "0.1.0-alpha".parse::<SemVer>().unwrap(),
            PRECEDENCE_STRINGS
                .into_iter()
                .map(|x| x.parse::<SemVer>().unwrap())
                .min()
                .unwrap()
        );
    }
    #[test]
    fn order() {
        let basic = PRECEDENCE_STRINGS
            .into_iter()
            .map(|x| x.parse::<SemVer>().unwrap())
            .collect::<Vec<SemVer>>();
        let mut rev = basic.clone();
        rev.reverse();
        assert_ne!(basic, rev);
        rev.sort();
        assert_eq!(basic, rev);
        let will_slice = rev
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>();
        assert_eq!(PRECEDENCE_STRINGS, will_slice.as_slice());
    }
}
