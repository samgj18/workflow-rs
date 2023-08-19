use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, SetSize},
};
use inquire::Select;

use crate::{
    domain::{command::Command, error::Error, workflow::Workflow},
    prelude::{Unit, WORKDIR},
};

use super::prelude::Parser;

pub trait Executor<T, R> {
    /// Given a `Workflow` struct, execute the workflow
    ///
    /// # Arguments
    /// * `workflow` - A `Workflow` struct
    ///
    /// # Returns
    /// * A `Result` with a `String` or an `Error`
    fn execute(&self, args: Option<T>) -> Result<R, Error>;
}

impl Executor<Workflow, Unit> for Command {
    fn execute(&self, workflow: Option<Workflow>) -> Result<Unit, Error> {
        match workflow {
            Some(workflow) => {
                match self {
                    Command::Run(_) => {
                        let precedence = workflow.try_parse::<Error>(None)?;
                        let command = workflow.command().replace(&precedence)?;
                        let (cols, rows) =
                            terminal::size().map_err(|e| Error::Io(Some(e.into())))?;
                        let text = format!(
                            "{}{}{}{}",
                            SetForegroundColor(Color::Green), // Set the text color to red
                            "Command to execute: ",
                            command,
                            ResetColor // Reset the text color to default
                        );

                        println!("\n");
                        println!("{}", text);
                        println!("\n");

                        let is_execute =
                            Select::new("Do you want to execute the command?", vec!["y", "n"])
                                .prompt()
                                .map(|s| s == "y")
                                .map_err(|e| Error::ReadError(Some(e.into())))?;

                        if is_execute {
                            execute!(
                                std::io::stdout(),
                                Clear(ClearType::All),
                                Print(text),
                                Print("\n"),
                                Print(command),
                                Print("\n"),
                            )
                            .map_err(|e| Error::Io(Some(e.into())))?;

                            execute!(std::io::stdout(), SetSize(cols, rows),)
                                .map_err(|e| Error::Io(Some(e.into())))?;
                        }

                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
            None => match self {
                // TODO: Print workflow descriptions
                Command::List(command) => {
                    let location = command.location().unwrap_or(&WORKDIR);
                    let files =
                        std::fs::read_dir(location).map_err(|e| Error::Io(Some(e.into())))?;

                    files
                        .into_iter()
                        .try_for_each::<_, Result<Unit, Error>>(|file| {
                            let file = file.map_err(|e| Error::Io(Some(e.into())))?;
                            let path = file.path().display().to_string();
                            let text = format!(
                                "{}{}{}{}",
                                SetForegroundColor(Color::Green), // Set the text color to red
                                "- ",
                                path,
                                ResetColor,
                            );

                            println!("{}", text);
                            Ok(())
                        })?;

                    Ok(())
                }
                _ => Err(Error::InvalidCommand(Some(
                    "Please provide a command. See --help".into(),
                ))),
            },
        }
    }
}
