use clap::Parser;

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
    #[cfg(test)]
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

#[derive(Parser, Debug)]
#[command(about = "Search for workflows, e.g. `workflow search <query>`")]
pub struct Search {
    #[arg(short, long, help = "The query to search for")]
    query: String,
}

impl Search {
    #[cfg(test)]
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }
}

#[derive(Parser, Debug, Default)]
#[command(about = "Remove existing index, e.g. `workflow index clean`")]
pub struct Clean {}
