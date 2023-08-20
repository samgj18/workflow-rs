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
    #[arg(short, long, help = "The location of the workflows")]
    location: Option<String>,
}

impl List {
    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }
}

#[derive(Parser, Debug)]
#[command(about = "Run a workflow, e.g. `workflow run <name>`")]
pub struct Run {
    #[arg(short, long, help = "The name of the workflow")]
    name: String,
    #[arg(short, long, help = "The location of the workflows")]
    location: Option<String>,
}

impl Run {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }
}
