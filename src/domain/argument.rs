use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ArgumentName(String);

impl ArgumentName {
    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArgumentDescription(String);

#[derive(Debug, Deserialize, Serialize)]
pub struct ArgumentDefault(String);

#[derive(Debug, Deserialize, Serialize)]
pub struct Argument {
    /// The name of the argument
    name: ArgumentName,
    /// The description of the argument
    description: Option<ArgumentDescription>,
    /// The default value of the argument
    #[serde(rename = "default_value")]
    default: Option<ArgumentDefault>,
}

impl Argument {
    pub fn name(&self) -> &ArgumentName {
        &self.name
    }

    pub fn description(&self) -> Option<&ArgumentDescription> {
        self.description.as_ref()
    }

    pub fn default(&self) -> Option<&ArgumentDefault> {
        self.default.as_ref()
    }

    /// Returns a map of the arguments with the default values
    pub fn parsed(&self, precedence: &HashMap<&str, &str>) -> Option<HashMap<String, String>> {
        let mut arguments = HashMap::new();
        let default_value = self
            .default()
            .unwrap_or(&ArgumentDefault("<insert value>".into()))
            .0
            .to_string();

        let name = &self.name().0;
        arguments.insert(name.to_owned(), default_value);

        if let Some(value) = precedence.get(name.as_str()) {
            arguments.insert(name.to_owned(), value.to_string());
        }

        match arguments {
            arguments if arguments.is_empty() => None,
            arguments => Some(arguments),
        }
    }
}
