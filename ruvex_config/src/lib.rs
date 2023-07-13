use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Config {
    pub cc_types: Vec<String>,
    pub minor_trigger: Vec<String>,
    pub patch_trigger: Vec<String>,
    pub check: Option<Check>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Check {
    pub name: Option<String>,
    pub diff: Option<String>,
}

impl Config {
    pub fn new(config_path: &str, default_config_path: &str) -> anyhow::Result<Config> {
        if std::fs::metadata(config_path).is_ok() {
        } else if default_config_path == config_path {
            return Err(anyhow::Error::msg("default path is not a real path"));
        } else {
            return Err(anyhow::Error::msg("provided path: {}\n does NOT exists"));
        }
        // Open the file in read-only mode with buffer.
        let file = std::fs::File::open(config_path)?;
        let reader = std::io::BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let config: Self = serde_yaml::from_reader(reader)?;

        // Return the `User`.
        Ok(config)
    }

    pub fn config_check(&self) -> anyhow::Result<()> {
        // check if all items in minor_trigger are in cc_types
        if !(self.minor_trigger.iter().all(|x| self.cc_types.contains(x))) {
            return Err(anyhow::Error::msg(
                format!(
                "\nConfig Error:\nall items of minor_trigger {:?} must be included in cc_types {:?}",
                self.minor_trigger, self.cc_types,
            )
                .red(),
            ));
        // check if all items in patch_trigger are in cc_types
        } else if !(self.patch_trigger.iter().all(|x| self.cc_types.contains(x))) {
            return Err(anyhow::Error::msg(
                format!(
                "\nConfig Error:\nall items of patch_trigger {:?} must be included in cc_types {:?}",
                self.patch_trigger, self.cc_types,
            )
                .red(),
            ));
        } else {
            return Ok(());
        }
    }

    pub fn write_default(path: &str) {
        let f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .expect(&format!("Error, couldn't open {}", path));
        serde_yaml::to_writer(f, &Self::default()).unwrap()
    }

    pub fn cc_type_in_config(&self, to_check: &str) -> bool {
        let mut commit_type_is_cc: bool = false;
        for cc_check in &self.cc_types {
            if to_check == cc_check {
                commit_type_is_cc = true;
            }
        }
        commit_type_is_cc
    }
}

impl Default for Config {
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
            check: Some(Check {
                name: None,
                diff: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Config;

    #[test]
    fn missing_minor_trigger_from_cc_types() {
        let test_config: Config = Config {
            cc_types: vec![
                "feat".to_owned(),
                "fix".to_owned(),
                "ci".to_owned(),
                "chore".to_owned(),
            ],
            minor_trigger: vec!["INEXISTENT".to_owned()],
            patch_trigger: vec!["fix".to_owned()],
            check: None,
        };
        assert!(test_config.config_check().is_err());
    }
    #[test]
    fn missing_patch_trigger_from_cc_types() {
        let test_config: Config = Config {
            cc_types: vec![
                "feat".to_owned(),
                "fix".to_owned(),
                "ci".to_owned(),
                "chore".to_owned(),
            ],
            minor_trigger: vec!["feat".to_owned()],
            patch_trigger: vec!["INEXISTENT".to_owned()],
            check: None,
        };
        assert!(test_config.config_check().is_err());
    }
}
