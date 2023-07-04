use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Config {
    pub cc_types: Vec<String>,
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
        } else {
            if default_config_path == config_path {
                return Err(anyhow::Error::msg("default path is not a real path"));
            } else {
                return Err(anyhow::Error::msg("provided path: {}\n does NOT exists"));
            }
        }
        // Open the file in read-only mode with buffer.
        let file = std::fs::File::open(config_path)?;
        let reader = std::io::BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let config = serde_yaml::from_reader(reader)?;

        // Return the `User`.
        Ok(config)
    }

    pub fn write_default(path: &str) {
        let f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .expect(&format!("Error, couldn't open {}", path));
        serde_yaml::to_writer(f, &Self::default()).unwrap()
    }

    pub fn default() -> Self {
        return Self {
            cc_types: vec![
                "feat".to_owned(),
                "fix".to_owned(),
                "ci".to_owned(),
                "chore".to_owned(),
            ],
            check: Some(Check {
                name: None,
                diff: None,
            }),
        };
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
