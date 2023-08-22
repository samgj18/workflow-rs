use std::{collections::HashMap, str::FromStr};

use crate::prelude::Error;

pub struct Configuration {
    workflow_dir: String,
}

impl Configuration {
    pub fn workflow_dir(&self) -> &str {
        &self.workflow_dir
    }
}

impl FromStr for Configuration {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let splitted = input.split('\n').collect::<Vec<&str>>();
        let mut configurations = HashMap::new();
        splitted.iter().for_each(|line| {
            if line.starts_with('#') {
                return;
            }

            let configuration = line.trim().split('=').collect::<Vec<&str>>();
            if configuration.len() != 2 {
                return;
            }

            let key = configuration[0];
            let value = configuration[1];

            configurations.insert(key, value);
        });

        let workflow_dir = configurations
            .get("workflow_dir")
            .map(|value| Ok(value.to_string()))
            .unwrap_or_else(|| {
                Err(Error::InvalidConfiguration(Some(
                    "Failed to read workflow_dir from configuration file".into(),
                )))
            })?;

        Ok(Self { workflow_dir })
    }
}
