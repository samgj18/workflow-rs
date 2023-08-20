use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub enum Command {
    Run(Run),
    List(List),
}

#[derive(Parser, Debug)]
#[command(about = "List all available workflows, e.g. `workflow list`")]
pub struct List {
    #[arg(
        short,
        long,
        help = "The location of the workflows: [default: $HOME/.workflow]"
    )]
    location: Option<String>,
}

impl List {
    #[cfg(test)]
    pub fn new(location: Option<&str>) -> Self {
        Self {
            location: location.map(|s| s.to_string()),
        }
    }

    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }
}

#[derive(Parser, Debug)]
#[command(about = "Run a workflow, e.g. `workflow run <name>`")]
pub struct Run {
    #[arg(short, long, help = "The name of the workflow")]
    name: String,
    #[arg(
        short,
        long,
        help = "The location of the workflows: [default: $HOME/.workflow]"
    )]
    location: Option<String>,
}

impl Run {
    #[cfg(test)]
    pub fn new(name: &str, location: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            location: location.map(|s| s.to_string()),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }
}
