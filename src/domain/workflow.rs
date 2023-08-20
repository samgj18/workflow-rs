use std::{cmp::Ordering, collections::HashMap};

use super::{args::Argument, prelude::Error};
use handlebars::Handlebars;
use inquire::{autocompletion::Replacement, Autocomplete, CustomUserError};
use serde::{Deserialize, Serialize};

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

impl WorkflowDescription {
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
        let handlebars = Handlebars::new();

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

    pub fn name(&self) -> &WorkflowName {
        &self.name
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

    pub fn values(&self) -> Vec<String> {
        self.arguments
            .iter()
            .flat_map(|argument| argument.values())
            .map(|value| value.inner().to_owned())
            .collect::<Vec<_>>()
    }
    pub(self) fn suggestion(&self, input: &str) -> Vec<String> {
        // Find the values with the minimum levenshtein distance
        let mut min_distance = usize::MAX;
        let mut min_values = Vec::new();

        for value in self.values() {
            let distance = Workflow::levenshtein(input, value.as_str());

            match distance.cmp(&min_distance) {
                Ordering::Less => {
                    min_distance = distance;
                    min_values = vec![value];
                }
                Ordering::Equal => {
                    min_values.push(value);
                }
                Ordering::Greater => {} // No action needed in this case
            }
        }

        min_values
    }

    pub(self) fn completion(&self, input: &str) -> Option<String> {
        // Find the value with the minimum levenshtein distance
        let mut min_distance = usize::MAX;
        let mut min_value = None;

        for value in self.values() {
            let distance = Workflow::levenshtein(input, value.as_str());

            if distance < min_distance {
                min_distance = distance;
                min_value = Some(value);
            }
        }

        min_value
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
}

impl Autocomplete for Workflow {
    /// Is called whenever the user's text input is modified
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        Ok(self.suggestion(input))
    }

    /// Is called whenever the user presses tab
    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        match highlighted_suggestion {
            Some(suggestion) => Ok(Replacement::Some(suggestion)),
            None => {
                let completion = self.completion(input);

                match completion {
                    Some(completion) => Ok(Replacement::Some(completion)),
                    None => Ok(Replacement::None),
                }
            }
        }
    }
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

        let suggestions = workflow.suggestion("test");

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
        let suggestions = workflow.suggestion("erg");

        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0], "tergiversation");
        assert_eq!(suggestions[1], "mergitramation");
    }
}
