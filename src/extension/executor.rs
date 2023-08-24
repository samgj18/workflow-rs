use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, SetSize},
};
use inquire::{required, Select, Text};

use crate::{
    domain::{command::Command, error::Error, workflow::Workflow},
    prelude::{prepare_workflows, Output, Prepare, Run, Unit, WorkflowDescription, WORKDIR},
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
                                Print("\n"),
                            )
                            .map_err(|e| Error::Io(Some(e.into())))?;

                            std::process::Command::new("sh")
                                .arg("-c")
                                .arg(&command)
                                .spawn()
                                .map_err(|e| Error::Io(Some(e.into())))?;

                            execute!(std::io::stdout(), SetSize(cols, rows),)
                                .map_err(|e| Error::Io(Some(e.into())))?;
                        }
                        Ok(Output::new("command", &command))
                    }
                    Command::List(_) => Ok(Output::unsupported()),
                    Command::Search(_) => Ok(Output::unsupported()),
                }
            }
            None => match self {
                Command::List(_) => {
                    let files =
                        std::fs::read_dir(&*WORKDIR).map_err(|e| Error::Io(Some(e.into())))?;

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
                            let path = file
                                .path()
                                .into_os_string()
                                .into_string()
                                .map_err(|_| Error::Io(None))?;

                            // Read the file and gather descriptions
                            let name = {
                                #[cfg(target_os = "windows")]
                                {
                                    path.split('\\').last()
                                }
                                #[cfg(not(target_os = "windows"))]
                                {
                                    path.split('/').last()
                                }
                            };
                            // Convert option to HashSet
                            let name = name.map_or_else(Vec::new, |name| vec![name]);

                            let workflows: Vec<String> = prepare_workflows(&name, &WORKDIR)?
                                .into_iter()
                                .map(|workflow| {
                                    let description = workflow
                                        .description()
                                        .map(|description| description.to_owned())
                                        .unwrap_or(WorkflowDescription::new("No description"));
                                    let name = workflow.name();
                                    let command = workflow.command();

                                    let description =
                                        format!("Description: {}", description.inner());
                                    let name = format!(
                                        "{}{}{}: ",
                                        SetForegroundColor(Color::Green),
                                        name.inner(),
                                        ResetColor
                                    );
                                    let command = format!("Command: {}", command.inner());

                                    format!(
                                        "* {}{}\n{}\n{}{}{}",
                                        SetForegroundColor(Color::White),
                                        name,
                                        description,
                                        command,
                                        "\n",
                                        ResetColor,
                                    )
                                })
                                .collect();

                            paths.push(path);

                            println!("{}", workflows.join("\n"));
                            Ok(())
                        })?;

                    Ok(Output::new("list", &paths.join("\n")))
                }
                Command::Run(_) => Err(Error::InvalidCommand(Some(
                    "Please provide a workflow. See --help".into(),
                ))),
                Command::Search(command) => {
                    let workflow = Text::new("Search for a workflow")
                        .with_validator(required!("This field is required"))
                        .with_autocomplete(command.clone())
                        .prompt()
                        .map_err(|e| Error::ReadError(Some(e.into())))?;

                    let workflow = workflow.trim().to_string();

                    let command = Command::Run(Run::new(&workflow));
                    let args = command.prepare()?;
                    command.execute(args)
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::List;

    pub const WORKFLOW: &str = {
        #[cfg(target_os = "windows")]
        {
            ".\\specs"
        }
        #[cfg(not(target_os = "windows"))]
        {
            "./specs"
        }
    };

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

    fn set_env_var() {
        #[cfg(target_os = "windows")]
        {
            std::env::set_var("WORKFLOW_DIR", WORKFLOW.replace("/", "\\"));
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::env::set_var("WORKFLOW_DIR", WORKFLOW);
        }
    }

    #[test]
    fn test_execute_list() {
        set_env_var();

        let command = Command::List(List::default());

        let result = command.execute(None).unwrap();
        let message = result.message();
        let r#type = result.r#type();

        let msg_res = {
            #[cfg(target_os = "windows")]
            {
                ".\\specs\\echo.yml"
            }
            #[cfg(not(target_os = "windows"))]
            {
                "./specs/echo.yml"
            }
        };

        assert_eq!(message, msg_res);
        assert_eq!(r#type, "list");
    }
}
