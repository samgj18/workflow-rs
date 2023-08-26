use std::sync::Arc;

use crossterm::{
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, Clear, ClearType, SetSize},
};
use inquire::Select;
use skim::{
    prelude::{unbounded, SkimOptionsBuilder},
    Skim, SkimItemReceiver, SkimItemSender,
};

use crate::{
    domain::{error::Error, workflow::Workflow},
    prelude::{List, Output, Prepare, Reset, Run, Search, Store, Unit, WorkflowDescription, STORE},
};

use super::prelude::Parser;

pub trait Executor {
    /// The error type
    type Error;
    /// The output type
    type Output;
    /// The arguments type
    type Args;

    /// Given a `Workflow` struct, execute the workflow
    ///
    /// # Arguments
    /// * `workflow` - A `Workflow` struct
    ///
    /// # Returns
    /// * A `Result` with a `String` or an `Error`
    fn execute(&self, args: Self::Args) -> Result<Self::Output, Self::Error>;
}

impl Executor for Run {
    type Error = Error;
    type Output = Output;
    type Args = Workflow;

    fn execute(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let precedence = args.try_parse::<Error>(None)?;
        let command = args.command().replace(&precedence)?;
        let (cols, rows) = terminal::size().map_err(|e| Error::Io(Some(e.into())))?;
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

        let is_execute = Select::new("Do you want to execute the command?", vec!["y", "n"])
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

            // What if the command needs a user input?
            let output: std::process::Output = std::process::Command::new("sh")
                .arg("-c")
                .arg(&command)
                .output()
                .map_err(|e| Error::Io(Some(e.into())))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let is_success = output.status.success();

            let message = if is_success {
                format!(
                    "{}{}{}",
                    SetForegroundColor(Color::Green),
                    stdout,
                    ResetColor
                )
            } else {
                format!("{}{}{}", SetForegroundColor(Color::Red), stderr, ResetColor)
            };

            execute!(std::io::stdout(), Print(message)).map_err(|e| Error::Io(Some(e.into())))?;

            execute!(std::io::stdout(), SetSize(cols, rows),)
                .map_err(|e| Error::Io(Some(e.into())))?;
        }
        Ok(Output::new("command", &command))
    }
}

impl Executor for List {
    type Error = Error;

    type Output = Output;

    type Args = Unit;

    fn execute(&self, _: Self::Args) -> Result<Self::Output, Self::Error> {
        let workflows = STORE.get_all();

        let workflows: Vec<String> = workflows?
            .into_iter()
            .map(|workflow| {
                let description = workflow
                    .description()
                    .map(|description| description.to_owned())
                    .unwrap_or(WorkflowDescription::new("No description"));
                let name = workflow.name();
                let command = workflow.command();

                let description = format!("Description: {}", description.inner());
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

        println!("{}", workflows.join("\n"));
        Ok(Output::new("list", "success"))
    }
}

impl Executor for Search {
    type Error = Error;
    type Output = Output;
    type Args = Unit;

    fn execute(&self, _: Self::Args) -> Result<Self::Output, Self::Error> {
        // TODO: Figure out how to also look by tags, author, etc.
        let workflows = STORE.get_all()?;

        let options = SkimOptionsBuilder::default()
            .height(Some("100%"))
            .multi(false)
            .preview(Some("")) // preview should be specified to enable preview window
            .build()
            .map_err(|e| Error::ReadError(Some(e.into())))?;

        let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
        let _ = std::thread::spawn(move || {
            workflows.into_iter().for_each(|workflow| {
                let _ = tx_item.send(Arc::new(workflow));
            });
        });

        let items = Skim::run_with(&options, Some(rx_item))
            .map(|out| out.selected_items)
            .unwrap_or_else(Vec::new);

        let selection = items
            .into_iter()
            .map(|item| item.clone().output().into_owned().trim().to_string())
            .filter(|item| !item.is_empty())
            .collect::<Vec<String>>();

        let workflow = selection
            .first()
            .ok_or_else(|| Error::ReadError(Some("No workflow selected".into())))?;

        let command = Run::new(workflow);
        let args = command.prepare()?;
        command.execute(args)
    }
}

impl Executor for Reset {
    type Error = Error;
    type Output = Output;
    type Args = Unit;

    fn execute(&self, _: Self::Args) -> Result<Self::Output, Self::Error> {
        execute!(
                        std::io::stdout(),
                        Clear(ClearType::All),
                        SetForegroundColor(Color::Green),
                        SetAttribute(Attribute::Bold),
                        Print("\u{2139} "),
                        SetAttribute(Attribute::Reset),
                        Print("The CLI does a reset every time you run it. This is to ensure that the workflows are up to date."),
                        Print("\n"),
                        Print("However, if you want to reset the workflows, you can do it here."),
                        Print("\n"),
                        Print("\n"),
                        SetForegroundColor(Color::Reset),
                    )
                    .map_err(|e| Error::Io(Some(e.into())))?;
        let is_reset = Select::new("Do you want to reset the workflows?", vec!["y", "n"])
            .prompt_skippable()
            .map(|s| s.map(|s| s == "y").unwrap_or(false))
            .map_err(|e| Error::ReadError(Some(e.into())))?;

        if is_reset {
            STORE.clone().delete_all()?;
            Ok(Output::new("reset", ""))
        } else {
            Ok(Output::new("reset", "No workflows were reset"))
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

        let command = List::default();

        let result = command.execute(()).unwrap();
        let message = result.message();
        let r#type = result.r#type();

        assert_eq!(message, "success");
        assert_eq!(r#type, "list");
    }
}
