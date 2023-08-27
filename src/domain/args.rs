use std::{fmt::Display, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::prelude::{Error, RawVec};

#[derive(Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
pub struct ArgumentName(String);

impl From<&str> for ArgumentName {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

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

impl Display for ArgumentValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}`", self.0)
    }
}

impl FromStr for ArgumentValue {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl ArgumentValue {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl FromStr for RawVec<ArgumentValue> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tags = s
            .split(',')
            .map(|tag| tag.trim().parse::<ArgumentValue>())
            .collect::<Result<Vec<ArgumentValue>, Error>>()?;
        Ok(RawVec::new(tags))
    }
}

impl Display for RawVec<ArgumentValue> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tags = self
            .tags()
            .iter()
            .map(|tag| tag.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{}", tags)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash, PartialEq, Eq)]
pub struct Argument {
    /// The name of the argument
    name: ArgumentName,
    /// The description of the argument
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<ArgumentDescription>,
    /// The default value of the argument
    #[serde(rename = "default_value", skip_serializing_if = "Option::is_none")]
    default: Option<ArgumentDefault>,
    /// The values that the argument can take
    #[serde(default = "Vec::new")]
    values: Vec<ArgumentValue>,
}

impl Argument {
    pub fn new(
        name: &str,
        description: Option<&str>,
        default: Option<&str>,
        values: Vec<ArgumentValue>,
    ) -> Self {
        Self {
            name: ArgumentName(name.to_string()),
            description: description.map(|d| ArgumentDescription(d.to_string())),
            default: default.map(|d| ArgumentDefault(d.to_string())),
            values,
        }
    }

    #[cfg(test)]
    pub fn skinny(name: &str, default: Option<&str>, values: Vec<&str>) -> Self {
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
