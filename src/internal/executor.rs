use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, SetSize},
};
use inquire::Select;

use crate::{
    domain::{command::Command, error::Error, workflow::Workflow},
    prelude::{Crawler, File, Output, Unit, INDEX_DIR, WORKDIR, WRITER},
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

impl Executor<Workflow, Output> for Command {
    fn execute(&self, workflow: Option<Workflow>) -> Result<Output, Error> {
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
                                .prompt_skippable()
                                .map(|s| s.map(|s| s == "y").unwrap_or(false))
                                .map_err(|e| Error::ReadError(Some(e.into())))?;

                        if is_execute {
                            execute!(
                                std::io::stdout(),
                                Clear(ClearType::All),
                                Print(text),
                                Print("\n"),
                                Print(command.clone()),
                                Print("\n"),
                            )
                            .map_err(|e| Error::Io(Some(e.into())))?;

                            execute!(std::io::stdout(), SetSize(cols, rows),)
                                .map_err(|e| Error::Io(Some(e.into())))?;
                        }
                        Ok(Output::new("command", &command))
                    }
                    Command::List(_) => Ok(Output::unsupported()),
                    Command::Scan(_) => Ok(Output::unsupported()),
                    Command::Clean(_) => Ok(Output::unsupported()),
                    Command::Search(_) => Ok(Output::unsupported()),
                }
            }
            None => match self {
                Command::List(command) => {
                    let location = command.location().unwrap_or(&WORKDIR);
                    let files =
                        std::fs::read_dir(location).map_err(|e| Error::Io(Some(e.into())))?;

                    let mut paths = vec![];

                    files
                        .into_iter()
                        .filter(|file| {
                            file.as_ref().map_or(false, |file| {
                                file.metadata().map_or(false, |metadata| metadata.is_file())
                            })
                        })
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

                            paths.push(path);

                            println!("{}", text);
                            Ok(())
                        })?;

                    Ok(Output::new("list", &paths.join("\n")))
                }
                Command::Run(_) => Err(Error::InvalidCommand(Some(
                    "Please provide a workflow. See --help".into(),
                ))),
                Command::Scan(command) => {
                    let location = command.location().unwrap_or(&WORKDIR);
                    let path = File::new(&format!("{}/{}", location, INDEX_DIR));

                    if path.exists() {
                        path.remove_all().and_then(|_| path.create_dir_all())?;
                    }

                    Crawler::crawl(location, &WRITER).map_err(|e| Error::Io(Some(e.into())))?;

                    let text = format!(
                        "{}{}{}{}",
                        SetForegroundColor(Color::Green), // Set the text color to red
                        "Scan created at ",
                        location,
                        ResetColor,
                    );

                    println!("{}", text);

                    Ok(Output::new(
                        "scan",
                        &format!("Scan created at {}", location),
                    ))
                }
                Command::Clean(_) => Err(Error::InvalidCommand(Some(
                    "Please provide a workflow. See --help".into(),
                ))),
                Command::Search(_) => Err(Error::InvalidCommand(Some(
                    "Please provide a workflow. See --help".into(),
                ))),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{List, Scan};

    pub const WORKFLOW: &str = "./specs";

    // Depends on https://github.com/mikaelmello/inquire/issues/70
    // #[test]
    // fn test_execute_run() {
    //     let command = Command::Run(Run::new("test", None));
    //     let workflow = Workflow::new("test", "ls", vec![]);
    //
    //     // Set timeout to send a ESC key to skip the prompt
    //     std::thread::sleep(std::time::Duration::from_secs(1));
    //
    //
    //     crossterm::event::poll(std::time::Duration::from_secs(1)).unwrap();
    //
    //     let result = command.execute(Some(workflow));
    //
    //     assert!(result.is_ok());
    // }

    #[test]
    fn test_execute_list() {
        let command = Command::List(List::new(Some(WORKFLOW)));

        let result = command.execute(None);
        let message = result.as_ref().unwrap().message();
        let r#type = result.as_ref().unwrap().r#type();
        let is_ok = result.is_ok();

        assert!(is_ok);
        assert_eq!(message, "./specs/echo.yml");
        assert_eq!(r#type, "list");
    }

    #[test]
    fn test_execute_scan() {
        let command = Command::Scan(Scan::new(Some(WORKFLOW)));

        let result = command.execute(None);
        let message = result.as_ref().unwrap().message();
        let r#type = result.as_ref().unwrap().r#type();
        let is_ok = result.is_ok();

        assert!(is_ok);
        assert_eq!(message, "Scan created at ./specs");
        assert_eq!(r#type, "scan");
    }
}
