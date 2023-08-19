use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArgumentName(String);

impl ArgumentName {
    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArgumentDescription(String);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArgumentDefault(String);

impl ArgumentDefault {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ArgumentValue(String);

impl ArgumentValue {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
    pub fn name(&self) -> &ArgumentName {
        &self.name
    }

    pub fn description(&self) -> Option<&ArgumentDescription> {
        self.description.as_ref()
    }

    pub fn default(&self) -> Option<&ArgumentDefault> {
        self.default.as_ref()
    }

    pub fn values(&self) -> &Vec<ArgumentValue> {
        &self.values
    }
}
