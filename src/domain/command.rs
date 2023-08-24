use clap::Parser;
use inquire::{autocompletion::Replacement, Autocomplete, CustomUserError};

use crate::prelude::{Fuzzy, Store, STORE};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub enum Command {
    Run(Run),
    List(List),
    Search(Search),
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

#[derive(Parser, Debug, Clone)]
#[command(about = "Search for workflows, e.g. `workflow search <query>`")]
pub struct Search {
    #[arg(short, long, help = "The query to search for")]
    query: Option<String>,
}

impl From<&str> for Search {
    fn from(query: &str) -> Self {
        Self {
            query: Some(query.to_string()),
        }
    }
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
        // Very simple without taking into account tags, description, etc.
        let suggestions = STORE
            .search()
            .into_iter()
            .flat_map(|workflows| {
                workflows
                    .into_iter()
                    .map(|workflow| workflow.name().inner().to_owned())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Ok(self
            .search(input, &suggestions)
            .into_iter()
            .map(|suggestion| suggestion.to_owned())
            .collect())
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
