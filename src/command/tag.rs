use log::debug;
use log::info;

use crate::command::check::check;
use crate::config::Config;
use crate::utils::git;
use crate::utils::semver::SemVer;
use crate::utils::CCVec;
use crate::utils::SemVerChangeType;
use std::process::Output;
// use ruvex_config::Config;

fn parse_tags(to_parse: &str) -> Vec<SemVer> {
    let default = SemVer {
        major: 0,
        minor: 0,
        patch: 0,
        pre_release: None,
        build_meta: None,
    };
    let tags: Vec<SemVer> = to_parse
        .split('\n')
        .map(|x| x.parse::<SemVer>().unwrap_or(default.clone()))
        .filter(|x| *x != default) // filter the 0.0.0
        .collect();
    tags
}

#[allow(dead_code)]
fn increase_semver(
    to_increase: SemVer,
    change: SemVerChangeType,
    prerelease: Option<String>,
    build_meta: Option<String>,
) -> SemVer {
    match change {
        SemVerChangeType::Major => increase_major(to_increase, prerelease, build_meta),
        SemVerChangeType::Minor => increase_minor(to_increase, prerelease, build_meta),
        SemVerChangeType::Patch => increase_patch(to_increase, prerelease, build_meta),
        SemVerChangeType::None => to_increase,
    }
}

#[allow(dead_code)]
fn increase_minor(
    to_increase: SemVer,
    prerelease: Option<String>,
    build_meta: Option<String>,
) -> SemVer {
    SemVer {
        major: to_increase.major + 1,
        minor: 0,
        patch: 0,
        pre_release: prerelease,
        build_meta,
    }
}
#[allow(dead_code)]
fn increase_major(
    to_increase: SemVer,
    prerelease: Option<String>,
    build_meta: Option<String>,
) -> SemVer {
    SemVer {
        major: to_increase.major + 1,
        minor: 0,
        patch: 0,
        pre_release: prerelease,
        build_meta,
    }
}
#[allow(dead_code)]
fn increase_patch(
    to_increase: SemVer,
    prerelease: Option<String>,
    build_meta: Option<String>,
) -> SemVer {
    SemVer {
        major: to_increase.major + 1,
        minor: 0,
        patch: 0,
        pre_release: prerelease,
        build_meta,
    }
}

fn latest_tag<'a>(
    tags: &'a [SemVer],
    ignore_prereleases: bool,
    config: &Config,
) -> Option<&'a SemVer> {
    match ignore_prereleases {
        true => tags.iter().filter(|x| x.pre_release.is_none()).max(),
        false => {
            match config.tag.is_some()
                && config.tag.as_ref().unwrap().ignore_prereleases.is_some()
                && config.tag.as_ref().unwrap().ignore_prereleases.unwrap()
            {
                true => tags.iter().filter(|x| x.pre_release.is_none()).max(),
                false => tags.iter().max(),
            }
        }
    }
}

pub fn tag(
    name: Option<Vec<String>>,
    merged: Option<String>,
    no_merged: Option<String>,
    ignore_prereleases: bool,
    config: &Config,
) -> anyhow::Result<()> {
    let out: Output = {
        if let Some(merged) = merged {
            git::tag(&["--merged", &merged])?
        } else if let Some(no_merged) = no_merged {
            git::tag(&["--no-merged", &no_merged])?
        } else {
            git::tag(&["-l"])?
        }
    };

    let binding = String::from_utf8(out.stdout)?;
    debug!("git tag command result is {:#?}", binding);

    let tags = parse_tags(&binding);
    info!("tags found are:");
    for tag in &tags {
        print!(" {}", tag);
    }

    let latest_tag = latest_tag(&tags, ignore_prereleases, config);

    let (good_commits, _bad_commits) = match (tags.is_empty(), name.is_some()) {
        (false, false) => check(
            Some(vec![format!("{}...", latest_tag.unwrap())]),
            None,
            config,
            false,
        )?,
        (true, false) => check(None, None, config, false)?,
        (false, true) => check(
            Some(
                name.unwrap()
                    .into_iter()
                    // if TAG keyword is found in name substitute it with latest_tag
                    .map(|x| x.replace("TAG", latest_tag.unwrap().to_string().as_str()))
                    .collect(),
            ),
            None,
            config,
            false,
        )?,
        (true, true) => check(name, None, config, false)?,
    };

    let semver_change = good_commits.max_change();

    let before_semver: SemVer;

    println!("Change type is: {:?}", semver_change);
    if let Some(tag) = latest_tag {
        println!("Latest identified SemVer tag is: {}", tag);
        before_semver = latest_tag.unwrap().clone();
    } else {
        println!("Latest identified SemVer tag is: None");
        before_semver = SemVer {
            major: 0,
            minor: 0,
            patch: 0,
            pre_release: None,
            build_meta: None,
        };
    }

    let current_semver: SemVer = match semver_change {
        SemVerChangeType::Major => SemVer {
            major: before_semver.major + 1,
            minor: 0,
            patch: 0,
            pre_release: None,
            build_meta: None,
        },
        SemVerChangeType::Minor => SemVer {
            major: before_semver.major,
            minor: before_semver.minor + 1,
            patch: 0,
            pre_release: None,
            build_meta: None,
        },
        SemVerChangeType::Patch => SemVer {
            major: before_semver.major,
            minor: before_semver.minor,
            patch: before_semver.patch + 1,
            pre_release: None,
            build_meta: None,
        },
        SemVerChangeType::None => before_semver,
    };
    println!("Next tag is {}", current_semver);

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::config::{Config, Tag};
    use crate::utils::semver::SemVer;

    use crate::command::tag::{latest_tag, parse_tags};
    #[test]
    fn test_parse() {
        let control = "0.1.0\ntest";
        let tags = parse_tags(control);
        let result = SemVer {
            major: 0,
            minor: 1,
            patch: 0,
            pre_release: None,
            build_meta: None,
        };
        assert!(tags.len() == 1);
        assert!(tags[0] == result);
    }
    #[test]
    fn test_parse_no_tags() {
        let control = "test\nababa\nnono";
        let tags = parse_tags(control);
        assert!(tags.is_empty());
        assert!(tags.iter().max().is_none());
        let control = "";
        let tags = parse_tags(control);
        assert!(tags.is_empty());
        assert!(tags.iter().max().is_none());
        let control = "0.0.0";
        let tags = parse_tags(control);
        assert!(tags.is_empty());
        assert!(tags.iter().max().is_none());
    }
    #[test]
    fn latest() {
        let mut config = Config::default();
        let mut test_vec = vec![
            SemVer {
                major: 0,
                minor: 1,
                patch: 0,
                pre_release: Some("a1".to_owned()),
                build_meta: None,
            },
            SemVer {
                major: 0,
                minor: 1,
                patch: 0,
                pre_release: None,
                build_meta: None,
            },
        ];

        assert_eq!(&test_vec[1], latest_tag(&test_vec, false, &config).unwrap());
        config.tag = Some(Tag {
            ignore_prereleases: None,
            merged: None,
            no_merged: None,
        });
        assert_eq!(&test_vec[1], latest_tag(&test_vec, false, &config).unwrap());

        test_vec.push(SemVer {
            major: 0,
            minor: 1,
            patch: 1,
            pre_release: Some("a1".to_owned()),
            build_meta: None,
        });
        assert_eq!(&test_vec[2], latest_tag(&test_vec, false, &config).unwrap());
        assert_eq!(&test_vec[1], latest_tag(&test_vec, true, &config).unwrap());
        config.tag = Some(Tag {
            ignore_prereleases: Some(true),
            merged: None,
            no_merged: None,
        });
        assert_eq!(&test_vec[1], latest_tag(&test_vec, false, &config).unwrap());
        config.tag = Some(Tag {
            ignore_prereleases: Some(false),
            merged: None,
            no_merged: None,
        });
        assert_eq!(&test_vec[2], latest_tag(&test_vec, false, &config).unwrap());
    }
}
