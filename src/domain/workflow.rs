use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    ops::Deref,
    str::FromStr,
};

use crate::prelude::Hasher;

use super::{args::Argument, prelude::Error};
use handlebars::Handlebars;
use inquire::CustomUserError;
use serde::{Deserialize, Serialize};
use strsim::normalized_levenshtein;

#[derive(Debug, Deserialize, Serialize, Clone, Hash, Eq, PartialEq)]
pub struct WorkflowName(String);

impl WorkflowName {
    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash, Eq, PartialEq)]
pub struct WorkflowDescription(String);

impl Deref for WorkflowDescription {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl WorkflowDescription {
    pub fn new(value: &str) -> Self {
        Self(value.into())
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash, Eq, PartialEq)]
pub struct WorkflowCommand(String);

impl WorkflowCommand {
    pub fn inner(&self) -> &str {
        &self.0
    }

    pub fn replace(&self, arguments: &HashMap<String, String>) -> Result<String, Error> {
        // Replace everything that is inside of {{}} with the value of the argument
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(|s| s.replace('\'', "\\'"));

        handlebars
            .render_template(&self.0, arguments)
            .map_err(|e| Error::ParseError(Some(e.into())))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash, Eq, PartialEq)]
pub struct WorkflowSource(String);

#[derive(Debug, Deserialize, Serialize, Clone, Hash, Eq, PartialEq)]
pub struct WorkflowAuthor(String);

#[derive(Debug, Deserialize, Serialize, Clone, Hash, Eq, PartialEq)]
pub struct WorkflowVersion(String);

#[derive(Debug, Deserialize, Serialize, Clone, Hash, Eq, PartialEq)]
pub struct WorkflowTag(String);

#[derive(Clone, Debug)]
pub struct RawVec<T>
where
    T: FromStr,
    T: Display,
    T: Clone,
{
    tags: Vec<T>,
}

impl<T> RawVec<T>
where
    T: FromStr,
    T: Display,
    T: Clone,
{
    pub fn new(tags: Vec<T>) -> Self {
        Self { tags }
    }

    pub fn inner(&self) -> &Vec<T> {
        &self.tags
    }

    pub fn into_inner(self) -> Vec<T> {
        self.tags
    }

    pub fn tags(&self) -> Vec<T> {
        self.tags.clone()
    }
}

impl<T> FromIterator<T> for RawVec<T>
where
    T: FromStr,
    T: Display,
    T: Clone,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let tags = iter.into_iter().collect::<Vec<T>>();
        Self { tags }
    }
}

impl FromStr for RawVec<WorkflowTag> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tags = s
            .split(',')
            .map(|tag| tag.trim().parse::<WorkflowTag>())
            .collect::<Result<Vec<WorkflowTag>, Error>>()?;
        Ok(RawVec { tags })
    }
}

impl Display for RawVec<WorkflowTag> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tags = self
            .tags
            .iter()
            .map(|tag| tag.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "{}", tags)
    }
}

impl FromStr for WorkflowTag {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl Display for WorkflowTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for WorkflowTag {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash, Eq, PartialEq)]
pub struct WorkflowId(String);

impl WorkflowId {
    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Hash)]
pub struct Workflow {
    /// The name of the workflow
    name: WorkflowName,
    /// The description of the workflow
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<WorkflowDescription>,
    /// The command to run the workflow
    command: WorkflowCommand,
    /// The commands to run the workflow
    #[serde(default = "Vec::new")]
    arguments: Vec<Argument>,
    /// The url to the source of the workflow
    #[serde(rename = "source_url", skip_serializing_if = "Option::is_none")]
    source: Option<WorkflowSource>,
    /// The author of the workflow
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<WorkflowAuthor>,
    /// The version of the workflow
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<WorkflowVersion>,
    /// The tags of the workflow
    #[serde(default = "Vec::new")]
    tags: Vec<WorkflowTag>,
}

impl ToString for Workflow {
    fn to_string(&self) -> String {
        format!("{:#?}", self)
    }
}

impl Workflow {
    pub fn new(
        name: &str,
        description: Option<&str>,
        command: &str,
        arguments: Vec<Argument>,
        source: Option<&str>,
        author: Option<&str>,
        tags: Vec<WorkflowTag>,
    ) -> Self {
        Self {
            name: WorkflowName(name.to_string()),
            description: description
                .map(|description| WorkflowDescription(description.to_string())),
            command: WorkflowCommand(command.to_string()),
            arguments,
            source: source.map(|source| WorkflowSource(source.to_string())),
            author: author.map(|author| WorkflowAuthor(author.to_string())),
            version: Some(WorkflowVersion("0.0.1".to_string())),
            tags,
        }
    }

    #[cfg(test)]
    pub fn slim(name: &str, command: &str, arguments: Vec<Argument>) -> Self {
        Self {
            name: WorkflowName(name.to_string()),
            description: None,
            command: WorkflowCommand(command.to_string()),
            arguments,
            source: None,
            author: None,
            version: None,
            tags: Vec::new(),
        }
    }

    pub fn id(&self) -> WorkflowId {
        WorkflowId(
            self.name
                .inner()
                .trim()
                .to_lowercase()
                .replace(['-', ' '], "_"),
        )
    }

    pub fn name(&self) -> &WorkflowName {
        &self.name
    }

    pub fn checksum(&self) -> u64 {
        Hasher::default().hash(&self.to_string())
    }

    pub fn description(&self) -> Option<&WorkflowDescription> {
        self.description.as_ref()
    }

    pub fn command(&self) -> &WorkflowCommand {
        &self.command
    }

    pub fn arguments(&self) -> &Vec<Argument> {
        &self.arguments
    }

    pub fn source(&self) -> Option<&WorkflowSource> {
        self.source.as_ref()
    }

    pub fn author(&self) -> Option<&WorkflowAuthor> {
        self.author.as_ref()
    }

    pub fn version(&self) -> Option<&WorkflowVersion> {
        self.version.as_ref()
    }

    pub fn tags(&self) -> &Vec<WorkflowTag> {
        &self.tags
    }

    pub fn values(&self) -> HashMap<String, Vec<String>> {
        self.arguments
            .iter()
            .map(|argument| {
                (
                    argument.name().inner().to_owned(),
                    argument
                        .values()
                        .iter()
                        .map(|value| value.inner().to_owned())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<HashMap<_, _>>()
    }

    pub fn suggestion(&self, input: &str, key: &str) -> Result<Vec<String>, CustomUserError> {
        let input = input.to_lowercase();

        Ok(self
            .values()
            .get(key)
            .unwrap_or(&Vec::new())
            .iter()
            .filter(|value| {
                value.to_lowercase().contains(&input)
                    || normalized_levenshtein(&input, value) >= 0.5
            })
            .take(5)
            .map(|value| value.to_owned())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provides_correct_simple_suggestions() {
        let arguments = vec![
            Argument::slim("test", Some("test"), vec!["test"]),
            Argument::slim("test2", Some("test2"), vec!["test2"]),
        ];
        let workflow = Workflow::slim("test", "test", arguments);

        let suggestions = workflow.suggestion("test", "test").unwrap();

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0], "test");
    }

    #[test]
    fn test_provides_correct_complex_suggestions() {
        let arguments = vec![Argument::slim(
            "test",
            Some("test"),
            vec![
                "surreptitious",
                "tergiversation",
                "mergitramation",
                "turreprosation",
            ],
        )];
        let workflow = Workflow::slim("test", "test", arguments);
        let suggestions = workflow.suggestion("erg", "test").unwrap();

        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0], "tergiversation");
        assert_eq!(suggestions[1], "mergitramation");
    }
}
