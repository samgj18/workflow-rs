use std::{collections::HashMap, ops::Deref};

use crate::prelude::Hasher;

use super::{args::Argument, prelude::Error};
use crossterm::style::{Attribute, Color, SetAttribute, SetForegroundColor};
use handlebars::Handlebars;
use inquire::CustomUserError;
use serde::{Deserialize, Serialize};
use skim::SkimItem;

const INSERTION_COST: usize = 1; // Weight for insertions
const DELETION_COST: usize = 1; // Weight for deletions
const SUBSTITUTION_COST: usize = 1; // Weight for substitutions

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkflowName(String);

impl WorkflowName {
    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkflowSource(String);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkflowAuthor(String);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkflowVersion(String);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkflowTag(String);

#[derive(Debug, Deserialize, Serialize, Clone, Hash, Eq, PartialEq)]
pub struct WorkflowId(String);

impl WorkflowId {
    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Workflow {
    /// The name of the workflow
    name: WorkflowName,
    /// The description of the workflow
    description: Option<WorkflowDescription>,
    /// The command to run the workflow
    command: WorkflowCommand,
    /// The commands to run the workflow
    #[serde(default = "Vec::new")]
    arguments: Vec<Argument>,
    /// The url to the source of the workflow
    #[serde(rename = "source_url")]
    source: Option<WorkflowSource>,
    /// The author of the workflow
    author: Option<WorkflowAuthor>,
    /// The version of the workflow
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
    #[cfg(test)]
    pub fn new(name: &str, command: &str, arguments: Vec<Argument>) -> Self {
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

    pub fn pretty_format(&self) -> String {
        // Search divides the terminal width by 2, so we do the same here

        let width = crossterm::terminal::size().unwrap_or((80, 20)).0 as usize / 2;
        let name = &self.name.inner();
        let available_width = width - name.len() - 3;

        let description = self.description.as_deref().unwrap_or("");
        let formatter = |description: &str| format!("\n{}\n", description);

        let limited_description = if description.len() > available_width {
            // Split the description into lines of the available width even if is
            // a one line description
            formatter(
                &description
                    .split_whitespace()
                    .fold((String::new(), 0), |(mut acc, mut current_length), word| {
                        let word_length = word.len();

                        if current_length + word_length >= available_width {
                            current_length = word_length;
                            acc.push_str(&format!("\n{}", word));
                        } else {
                            current_length += word_length;
                            acc.push_str(&format!(" {}", word));
                        }

                        (acc, current_length)
                    })
                    .0,
            )
        } else {
            formatter(description)
        };

        let command = &self.command().inner();
        let limited_command = if command.len() > available_width {
            formatter(
                &command
                    .split_whitespace()
                    .fold((String::new(), 0), |(mut acc, mut current_length), word| {
                        let word_length = word.len();

                        if current_length + word_length >= available_width {
                            current_length = word_length;
                            acc.push_str(&format!("\n{}", word));
                        } else {
                            current_length += word_length;
                            acc.push_str(&format!(" {}", word));
                        }

                        (acc, current_length)
                    })
                    .0,
            )
        } else {
            formatter(command)
        };

        format!(
            "{}{}{}{}\n\n{}* {}\n\n{}{}{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Green),
            name,
            SetForegroundColor(Color::Reset),
            SetAttribute(Attribute::Reset),
            limited_description.trim(),
            SetAttribute(Attribute::Italic),
            limited_command.trim(),
            SetAttribute(Attribute::Reset),
        )
    }

    pub fn suggestion(&self, input: &str, key: &str) -> Result<Vec<String>, CustomUserError> {
        let input = input.to_lowercase();

        Ok(self
            .values()
            .get(key)
            .unwrap_or(&Vec::new())
            .iter()
            .filter(|value| {
                value.to_lowercase().contains(&input) || levenshtein(&input, value) <= 2
            })
            .take(5)
            .map(|value| value.to_owned())
            .collect())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexedWorkflow {
    id: Vec<String>,
    body: Vec<Workflow>,
}

impl IndexedWorkflow {
    pub fn id(&self) -> String {
        self.body
            .first()
            .map(|workflow| workflow.id().inner().to_owned())
            .unwrap_or_default()
    }
}

impl SkimItem for Workflow {
    fn text(&self) -> std::borrow::Cow<str> {
        self.name.inner().into()
    }

    fn preview(&self, _context: skim::PreviewContext) -> skim::ItemPreview {
        skim::ItemPreview::AnsiText(self.pretty_format())
    }
}

/// Calculates the levenshtein distance between two strings
///
/// # Credit
///
/// Credit where credit is due. This implementation is taken from [wooorm/levenshtein-rs](https://github.com/wooorm/levenshtein-rs)
fn levenshtein(from: &str, to: &str) -> usize {
    let mut result = 0;

    if from == to {
        return result;
    }

    let length_a = from.chars().count();
    let length_b = to.chars().count();

    if length_a == 0 {
        return length_b;
    }

    if length_b == 0 {
        return length_a;
    }

    let mut cache: Vec<usize> = (1..).take(length_a).collect();
    let mut distance_a;
    let mut distance_b;

    for (index_b, code_b) in to.chars().enumerate() {
        result = index_b;
        distance_a = index_b;

        for (index_a, code_a) in from.chars().enumerate() {
            distance_b = if code_a == code_b {
                distance_a
            } else {
                distance_a + SUBSTITUTION_COST
            };

            distance_a = cache[index_a];

            result = if distance_a > result {
                if distance_b > result {
                    result + INSERTION_COST
                } else {
                    distance_b
                }
            } else if distance_b > distance_a {
                distance_a + DELETION_COST
            } else {
                distance_b
            };

            cache[index_a] = result;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provides_correct_simple_suggestions() {
        let arguments = vec![
            Argument::new("test", Some("test"), vec!["test"]),
            Argument::new("test2", Some("test2"), vec!["test2"]),
        ];
        let workflow = Workflow::new("test", "test", arguments);

        let suggestions = workflow.suggestion("test", "test").unwrap();

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0], "test");
    }

    #[test]
    fn test_provides_correct_complex_suggestions() {
        let arguments = vec![Argument::new(
            "test",
            Some("test"),
            vec![
                "surreptitious",
                "tergiversation",
                "mergitramation",
                "turreprosation",
            ],
        )];
        let workflow = Workflow::new("test", "test", arguments);
        let suggestions = workflow.suggestion("erg", "test").unwrap();

        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0], "tergiversation");
        assert_eq!(suggestions[1], "mergitramation");
    }
}
