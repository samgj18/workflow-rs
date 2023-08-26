use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
pub struct ArgumentName(String);

impl ArgumentName {
    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
pub struct ArgumentDescription(String);

impl Deref for ArgumentDescription {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<ArgumentDescription> for ArgumentDescription {
    fn as_ref(&self) -> &ArgumentDescription {
        self
    }
}

impl ArgumentDescription {
    pub fn new(value: &str) -> Self {
        Self(value.into())
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
pub struct ArgumentDefault(String);

impl ArgumentDefault {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
pub struct ArgumentValue(String);

impl ArgumentValue {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
pub struct Argument {
    /// The name of the argument
    name: ArgumentName,
    /// The description of the argument
    description: Option<ArgumentDescription>,
    /// The default value of the argument
    #[serde(rename = "default_value")]
    default: Option<ArgumentDefault>,
    /// The values that the argument can take
    #[serde(default = "Vec::new")]
    values: Vec<ArgumentValue>,
}

impl Argument {
    #[cfg(test)]
    pub fn new(name: &str, default: Option<&str>, values: Vec<&str>) -> Self {
        Self {
            name: ArgumentName(name.to_string()),
            description: None,
            default: default.map(|d| ArgumentDefault(d.to_string())),
            values: values
                .into_iter()
                .map(|v| ArgumentValue(v.to_string()))
                .collect(),
        }
    }

    pub fn name(&self) -> &ArgumentName {
        &self.name
    }

    pub fn description(&self) -> Option<&ArgumentDescription> {
        self.description.as_ref()
    }

    pub fn def_description(&self) -> &str {
        self.description.as_deref().unwrap_or("")
    }

    pub fn default(&self) -> Option<&ArgumentDefault> {
        self.default.as_ref()
    }

    pub fn values(&self) -> &Vec<ArgumentValue> {
        &self.values
    }
}
