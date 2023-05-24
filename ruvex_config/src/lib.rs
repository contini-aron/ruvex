use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Config {
    pub cc_types: Vec<String>,
}

impl Config {
    pub fn new(config_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
        // Open the file in read-only mode with buffer.
        let file = std::fs::File::open(config_path)?;
        let reader = std::io::BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let config = serde_yaml::from_reader(reader)?;

        // Return the `User`.
        Ok(config)
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
