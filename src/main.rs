use clap::Parser;
use workflow::prelude::*;

fn main() -> Result<Unit, Error> {
    // Crawls the current directory for workflow files
    Crawler::crawl(&WORKDIR, &WRITER)?;

    // Parse the command line arguments.
    let command: Command = Command::parse();

    // Prepare and execute the command.
    let workflow = command.prepare()?;
    command.execute(workflow)?;

    Ok(())
}
