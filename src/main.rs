use clap::Parser;
use workflow::prelude::*;

fn main() -> Result<Unit, Error> {
    // Crawls the current directory for workflow files
    Crawler::crawl(&WORKDIR, &STORE)?;

    // Parse the command line arguments.
    let command: Command = Command::parse();

    // Execute the command.
    match command {
        Command::Run(command) => {
            let workflow = command.prepare()?;
            command.execute(workflow)?;
        }
        Command::List(command) => {
            command.execute(())?;
        }
        Command::Search(command) => {
            command.execute(())?;
        }
        Command::Reset(command) => {
            command.execute(())?;
        }
    }

    Ok(())
}
