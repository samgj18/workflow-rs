use std::collections::HashMap;

use super::{argument::Argument, prelude::Error};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkflowName(String);

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkflowDescription(String);

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkflowSource(String);

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkflowAuthor(String);

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkflowVersion(String);

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkflowTag(String);

#[derive(Debug, Deserialize, Serialize)]
pub struct Workflow {
    /// The name of the workflow
    name: WorkflowName,
    /// The description of the workflow
    description: Option<WorkflowDescription>,
    /// The command to run the workflow
    command: WorkflowCommand,
    /// The commands to run the workflow
    arguments: Vec<Argument>,
    /// The url to the source of the workflow
    #[serde(rename = "source_url")]
    source: Option<WorkflowSource>,
    /// The author of the workflow
    author: Option<WorkflowAuthor>,
    /// The version of the workflow
    version: Option<WorkflowVersion>,
    /// The tags of the workflow
    tags: Vec<WorkflowTag>,
}

impl Workflow {
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

    /// Returns a map of the arguments with the default values or the values
    /// provided by the user in the workflow call
    pub fn parsed(&self, precedence: &HashMap<&str, &str>) -> HashMap<String, String> {
        let mut arguments = HashMap::new();
        self.arguments().iter().for_each(|arg| {
            if let Some(args) = arg.parsed(precedence) {
                arguments.extend(args);
            }
        });
        arguments
    }
}
