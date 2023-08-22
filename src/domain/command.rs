use clap::Parser;
use inquire::{autocompletion::Replacement, Autocomplete, CustomUserError};

use crate::{
    domain::workflow::IndexedWorkflow,
    prelude::{SearchTerm, SearchTermLimit, READER},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub enum Command {
    Run(Run),
    List(List),
    Search(Search),
    #[clap(subcommand)]
    Index(Indexer),
}

#[derive(Parser, Debug, Default)]
#[command(about = "List all available workflows, e.g. `workflow list`")]
pub struct List {}

#[derive(Parser, Debug)]
#[command(about = "Run a workflow, e.g. `workflow run <name>`")]
pub struct Run {
    #[arg(short, long, help = "The name of the workflow")]
    name: String,
}

impl Run {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Parser, Debug)]
#[command(about = "Index workflows, e.g. `workflow index <create|clean>`")]
pub enum Indexer {
    Clean(Clean),
    Create(Create),
}

impl Indexer {
    #[cfg(test)]
    pub fn new(command: &str) -> Self {
        match command {
            "clean" => Self::Clean(Clean::default()),
            "create" => Self::Create(Create::default()),
            _ => unreachable!(),
        }
    }
}

#[derive(Parser, Debug, Default)]
#[command(about = "Create a new index deleting existing one, e.g. `workflow index create`")]
pub struct Create {}

#[derive(Parser, Debug, Default)]
#[command(about = "Remove existing index, e.g. `workflow index clean`")]
pub struct Clean {}

#[derive(Parser, Debug, Clone)]
#[command(about = "Search for workflows, e.g. `workflow search <query>`")]
pub struct Search {
    #[arg(short, long, help = "The query to search for")]
    query: Option<String>,
}

impl Search {
    #[cfg(test)]
    pub fn new(query: Option<&str>) -> Self {
        Self {
            query: query.map(|s| s.to_string()),
        }
    }

    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }
}

impl Autocomplete for Search {
    /// Is called whenever the user's text input is modified
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        let fields = vec!["body"];
        let input = if input.is_empty() {
            "*".to_string()
        } else {
            format!(
                "body:{}* OR body.name:{}* OR body.description:{}* OR body.tags:{}* OR body.author:{}* OR body.version:{}*",
                input, input, input, input, input, input
            )
        };
        let term = SearchTerm::from(
            input.as_str(), // format!("body.name:{} OR body.description:{}", input, input,).as_str(),
        );
        let limit = SearchTermLimit::from(100);
        let workflow = READER
            .clone()
            .get_as::<IndexedWorkflow>(&term, &fields, Some(limit).as_ref())
            .map_err(|e| CustomUserError::from(e.to_string()));

        let mut suggestions = vec![];

        if let Ok(workflow) = workflow {
            for (_, workflow) in workflow {
                suggestions.push(workflow.name().to_string());
            }
        }

        Ok(suggestions)
    }

    /// Is called whenever the user presses tab
    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, CustomUserError> {
        match highlighted_suggestion {
            Some(suggestion) => Ok(Replacement::Some(suggestion)),
            None => {
                let completion = {
                    let suggestions = self.get_suggestions(input)?;
                    suggestions.first().map(|s| s.to_string())
                };

                match completion {
                    Some(completion) => Ok(Replacement::Some(completion)),
                    None => Ok(Replacement::None),
                }
            }
        }
    }
}
