use colored::Colorize;
use serde::{Deserialize, Serialize};

/// Top-level configuration for the versioning tool.
///
/// Loaded from a YAML file and controls which commit types are recognized,
/// which types bump the minor/patch version, and optional tag/check settings.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// All valid conventional commit types (e.g. "feat", "fix", "chore").
    /// `minor_trigger` and `patch_trigger` must be subsets of this list.
    pub cc_types: Vec<String>,

    /// Commit types that trigger a **minor** version bump (e.g. ["feat"]).
    pub minor_trigger: Vec<String>,

    /// Commit types that trigger a **patch** version bump (e.g. ["fix"]).
    pub patch_trigger: Vec<String>,

    /// Optional settings for diff/name checks during CI validation.
    pub check: Option<Check>,

    /// Optional settings that control how Git tags are resolved.
    pub tag: Option<Tag>,
}

/// Controls which Git tags are considered when determining the latest version.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tag {
    /// If set, only tags reachable from this merged branch are considered.
    pub merged: Option<String>,

    /// If set, only tags NOT merged into this branch are considered.
    pub no_merged: Option<String>,

    /// When `true`, pre-release tags (e.g. `v1.0.0-rc.1`) are excluded
    /// from the latest-version lookup.
    pub ignore_prereleases: Option<bool>,
}

/// Optional CI check configuration for validating commit content.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Check {
    /// Expected name pattern to validate against.
    pub name: Option<String>,

    /// Expected diff pattern to validate against.
    pub diff: Option<String>,
}

impl Config {
    /// Load a [`Config`] from `config_path`.
    ///
    /// Returns an error if:
    /// - `config_path` does not exist (with a hint to use `--create-default`)
    /// - the file cannot be opened or parsed as valid YAML
    pub fn new(config_path: &str, default_config_path: &str) -> anyhow::Result<Self> {
        if std::fs::metadata(config_path).is_err() {
            // Avoid a misleading error when the caller passes the same path for
            // both arguments (i.e. the default path itself doesn't exist yet).
            if default_config_path == config_path {
                return Err(anyhow::anyhow!("default path is not a real path"));
            }
            return Err(anyhow::anyhow!(
                "provided path: {}\ndoes NOT exist\n\
                 If you would like to generate a default config, pass a --create-default flag",
                config_path
            ));
        }

        let file = std::fs::File::open(config_path)?;
        let reader = std::io::BufReader::new(file);
        let config: Self = serde_yaml::from_reader(reader)?;

        Ok(config)
    }

    /// Validate that all entries in `minor_trigger` and `patch_trigger` are
    /// present in `cc_types`.
    ///
    /// This must be called after loading the config to catch user mistakes
    /// (e.g. a typo in a trigger type) before they silently affect versioning.
    pub fn config_check(&self) -> anyhow::Result<()> {
        // Helper closure: returns true only if every trigger exists in cc_types.
        let all_in_cc_types = |triggers: &[String]| {
            triggers.iter().all(|t| self.cc_types.contains(t))
        };

        if !all_in_cc_types(&self.minor_trigger) {
            return Err(anyhow::anyhow!(
                "\nConfig Error:\nall items of minor_trigger {:?} must be included in cc_types {:?}",
                self.minor_trigger,
                self.cc_types,
            )
            .context("".red().to_string()));
        }

        if !all_in_cc_types(&self.patch_trigger) {
            return Err(anyhow::anyhow!(
                "\nConfig Error:\nall items of patch_trigger {:?} must be included in cc_types {:?}",
                self.patch_trigger,
                self.cc_types,
            )
            .context("".red().to_string()));
        }

        Ok(())
    }

    /// Serialize the [`Default`] config and write it to `path`, creating or
    /// truncating the file as needed. Panics if the file cannot be opened.
    pub fn write_default(path: &str) {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .unwrap_or_else(|_| panic!("Error: couldn't open '{}'", path));

        serde_yaml::to_writer(file, &Self::default()).unwrap();
    }

    /// Returns `true` if `to_check` is listed as a valid commit type in `cc_types`.
    pub fn cc_type_in_config(&self, to_check: &str) -> bool {
        self.cc_types.iter().any(|cc| cc == to_check)
    }
}

impl Default for Config {
    /// Sensible out-of-the-box defaults:
    /// - recognises `feat`, `fix`, `ci`, and `chore` commit types
    /// - `feat` → minor bump, `fix` → patch bump
    /// - no check or tag filters applied
    fn default() -> Self {
        Self {
            cc_types: vec![
                "feat".to_owned(),
                "fix".to_owned(),
                "ci".to_owned(),
                "chore".to_owned(),
            ],
            minor_trigger: vec!["feat".to_owned()],
            patch_trigger: vec!["fix".to_owned()],
            check: Some(Check { name: None, diff: None }),
            tag: Some(Tag {
                merged: None,
                no_merged: None,
                ignore_prereleases: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    /// Baseline config used across tests — mirrors the `Default` impl
    /// but with `check` and `tag` set to `None` for simplicity.
    fn base_config() -> Config {
        Config {
            cc_types: vec![
                "feat".to_owned(),
                "fix".to_owned(),
                "ci".to_owned(),
                "chore".to_owned(),
            ],
            minor_trigger: vec!["feat".to_owned()],
            patch_trigger: vec!["fix".to_owned()],
            check: None,
            tag: None,
        }
    }

    /// `config_check` should reject a `minor_trigger` entry that isn't in `cc_types`.
    #[test]
    fn missing_minor_trigger_from_cc_types() {
        let config = Config {
            minor_trigger: vec!["INEXISTENT".to_owned()],
            ..base_config()
        };
        assert!(config.config_check().is_err());
    }

    /// `config_check` should reject a `patch_trigger` entry that isn't in `cc_types`.
    #[test]
    fn missing_patch_trigger_from_cc_types() {
        let config = Config {
            patch_trigger: vec!["INEXISTENT".to_owned()],
            ..base_config()
        };
        assert!(config.config_check().is_err());
    }
}
