use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, propagate_version = true)]
pub enum Command {
    Run(Run),
    List(List),
    Search(Search),
    Reset(Reset),
}

#[derive(Parser, Debug, Default)]
#[command(about = "List all available workflows, e.g. `workflow list`")]
pub struct List;

#[derive(Parser, Debug)]
#[command(about = "Run a workflow, e.g. `workflow run --name <name>`")]
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

#[derive(Parser, Debug, Clone, Default)]
#[command(about = "Search for workflows, e.g. `workflow search`")]
pub struct Search;

impl Search {
    #[cfg(test)]
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Parser, Debug, Default)]
#[command(about = "Reset the workflow store, e.g. `workflow reset`")]
pub struct Reset;

impl Reset {
    #[cfg(test)]
    pub fn new() -> Self {
        Self {}
    }
}
