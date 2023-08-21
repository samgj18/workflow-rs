use clap::Parser;
use workflow::prelude::*;

fn main() -> Result<Unit, Error> {
    // Create the index, reader and writer.
    let index = Index::new()?;
    // let reader = Reader::new(&index)?;
    let mut writer = Writer::new(&index)?;

    // Crawling the directory.
    Crawler::crawl(&WORKDIR, &mut writer)?;

    // Parse the command line arguments.
    let command: Command = Command::parse();
    // command.crawl(&WORKDIR, writer)?;

    // Prepare and execute the command.
    let workflow = command.prepare()?;
    command.execute(workflow)?;

    Ok(())
}
