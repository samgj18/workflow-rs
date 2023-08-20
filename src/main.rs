use clap::Parser;
use workflow::prelude::*;

fn main() -> Result<Unit, Error> {
    let command: Command = Command::parse();
    let workflow = command.prepare()?;
    command.execute(workflow)?;

    Ok(())
}
