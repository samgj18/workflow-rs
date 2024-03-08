use std::{cmp::Ordering, collections::HashSet, fs::OpenOptions};

use crossterm::{
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, Clear, ClearType, SetSize},
};
use inquire::{required, Confirm, CustomType, Text};
use strsim::normalized_levenshtein;

use crate::{
    domain::{error::Error, workflow::Workflow},
    prelude::{
        Argument, ArgumentValue, Create, List, Output, Prepare, RawVec, Reset, Run, Search, Store,
        Unit, WorkflowDescription, WorkflowTag, STORE, WORKDIR,
    },
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
        let precedence = args.try_parse(())?;
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

        let is_execute = Confirm::new("Do you want to execute the command?")
            .prompt()
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
        let threshold = 0.5; // Adjust the threshold as needed

        let names = workflows
            .into_iter()
            .map(|workflow| workflow.name().inner().to_owned())
            .collect::<Vec<String>>();

        let workflow = Text::new("Search for a workflow: ")
            .with_autocomplete(move |query: &str| {
                let mut similarity_scores =
                    names
                        .iter()
                        .fold::<Vec<(String, f64)>, _>(Vec::new(), |mut acc, value| {
                            let distance = normalized_levenshtein(query, value);
                            let similarity = 1.0 - distance;
                            acc.push((value.to_string(), similarity));

                            acc
                        });

                similarity_scores
                    .sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(Ordering::Equal));

                let suggestions = similarity_scores
                    .into_iter()
                    .rev()
                    .filter(|(_, score)| *score > threshold)
                    .map(|(value, _)| value)
                    .collect::<Vec<String>>();
                Ok(suggestions)
            })
            .prompt()
            .map_err(|e| Error::ReadError(Some(e.into())))?;

        let command = Run::new(&workflow);
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
        let is_reset = Confirm::new("Do you want to reset the workflows?")
            .prompt()
            .map_err(|e| Error::ReadError(Some(e.into())))?;

        if is_reset {
            STORE.clone().delete_all()?;
            Ok(Output::new("reset", ""))
        } else {
            Ok(Output::new("reset", "No workflows were reset"))
        }
    }
}

impl Executor for Create {
    type Error = Error;
    type Output = Output;
    type Args = Unit;

    fn execute(&self, _: Self::Args) -> Result<Self::Output, Self::Error> {
        let name: String = Text::new("What is the name of the workflow?")
            .with_help_message("This is the name that will be used to run the workflow")
            .with_validator(required!("Name is required"))
            .prompt()
            .map_err(|e| Error::ReadError(Some(e.into())))?;

        let description: Option<String> = Text::new("What is the description of the workflow?")
            .with_help_message("This is the description that will be used to describe the workflow")
            .prompt_skippable()
            .map_err(|e| Error::ReadError(Some(e.into())))?
            .filter(non_empty_filter);

        let command: String = Text::new("What is the command of the workflow?")
            .with_help_message("This is the command that will be executed when the workflow is run")
            .with_validator(required!("Command is required"))
            .prompt()
            .map_err(|e| Error::ReadError(Some(e.into())))?;

        let arguments: Vec<Argument> = arguments_builder(&command)?;

        let source: Option<String> = Text::new("What is the source of the workflow?")
            .with_help_message("This is the link, if any, to the source of the workflow")
            .prompt_skippable()
            .map_err(|e| Error::ReadError(Some(e.into())))?
            .filter(non_empty_filter);

        let author: Option<String> = Text::new("Who is the author of the workflow?")
            .with_help_message("This is the name of the author of the workflow")
            .prompt_skippable()
            .map_err(|e| Error::ReadError(Some(e.into())))?
            .filter(non_empty_filter);

        let tags: Vec<WorkflowTag> =
            CustomType::<RawVec<WorkflowTag>>::new("What are the tags of the workflow?")
                .with_help_message("Please enter a comma separated list of tags")
                .with_parser(&|input| {
                    let tags = input
                        .split(',')
                        .map(|tag| tag.trim().to_string())
                        .collect::<Vec<String>>();
                    Ok(tags
                        .into_iter()
                        .map(WorkflowTag::from)
                        .collect::<RawVec<WorkflowTag>>())
                })
                .prompt_skippable()
                .map_err(|e| Error::ReadError(Some(e.into())))?
                .map(|raw| raw.into_inner())
                .unwrap_or_else(Vec::new);

        let workflow: Workflow = Workflow::new(
            &name,
            description.as_deref(),
            &command,
            arguments,
            source.as_deref(),
            author.as_deref(),
            tags,
        );

        let path = WORKDIR.join(format!("{}.yml", name));

        let writer = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| Error::WriteError(Some(e.into())))?;

        serde_yaml::to_writer(&writer, &workflow).map_err(|e| Error::WriteError(Some(e.into())))?;

        execute!(
            std::io::stdout(),
            Clear(ClearType::All),
            SetForegroundColor(Color::Green),
            Print(format!("Workflow {} created at {}", name, path.display())),
            SetForegroundColor(Color::Reset)
        )
        .map_err(|e| Error::Io(Some(e.into())))?;

        Ok(Output::new("create", &format!("Workflow {} created", name)))
    }
}

fn non_empty_filter<T: ToString>(value: &T) -> bool {
    !value.to_string().trim().is_empty()
}

fn get_values(command: &str) -> Result<HashSet<&str>, Error> {
    let values = command
        .split("{{")
        .skip(1)
        .filter_map(|part| part.split("}}").next().map(|inner_part| inner_part.trim()))
        .filter(|part| !part.is_empty())
        .collect();

    Ok(values)
}

fn read_argument_value(question: &str, help: &str) -> Result<Option<String>, Error> {
    Ok(Text::new(question)
        .with_help_message(help)
        .prompt_skippable()
        .map_err(|e| Error::ReadError(Some(e.into())))?
        .filter(non_empty_filter))
}

fn read_example_values() -> Result<Vec<String>, Error> {
    let mut values = Vec::new();
    loop {
        let value = read_argument_value(
            "What is the example value?",
            "This is the value that will be used to show the user how to use the argument",
        )?;
        if let Some(value) = value {
            values.push(value);
        }
        let has_more_values = Confirm::new("Do you want to add more values")
            .prompt()
            .map_err(|e| Error::ReadError(Some(e.into())))?;
        if !has_more_values {
            break;
        }
    }
    Ok(values)
}

fn arguments_builder(command: &str) -> Result<Vec<Argument>, Error> {
    let is_ordered = command
        .match_indices("{{")
        .zip(command.match_indices("}}"))
        .all(|(start, end)| start < end);

    let open_count = command.matches("{{").count();
    let close_count = command.matches("}}").count();

    let is_balanced = open_count == close_count;
    let has_args = is_ordered && is_balanced && open_count != 0;

    if has_args {
        let values = get_values(command)?;

        let arguments = values
            .iter()
            .map(|arg| {
                let description = read_argument_value(
                    &format!("What is the description of the argument '{}'?", arg),
                    "This is the description that will be used as the help message for the argument",
                )?;
                let default = read_argument_value(
                    &format!("What is the default value of the argument '{}'?", arg),
                    "This is the default value that will be used if the user does not provide a value",
                )?;
                let has_example_values =
                    Confirm::new(&format!("Do you want to add example values for '{}'?", arg))
                        .with_help_message("This is the value that will be used to show to the user for suggestions")
                        .prompt()
                        .map_err(|e| Error::ReadError(Some(e.into())))?;
                let values = if has_example_values {
                    read_example_values()?
                        .into_iter()
                        .map(ArgumentValue::new)
                        .collect()
                } else {
                    Vec::new()
                };
                Ok(Argument::new(
                    arg,
                    description.as_deref(),
                    default.as_deref(),
                    values,
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(arguments)
    } else if is_ordered && !is_balanced {
        Err(Error::InvalidCommand(Some(
            "The command has an unbalanced number of {{ and }}".into(),
        )))
    } else {
        Ok(Vec::new())
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
    // as well as Search and Reset

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

        let command = List;

        let result = command.execute(()).unwrap();
        let message = result.message();
        let r#type = result.r#type();

        assert_eq!(message, "success");
        assert_eq!(r#type, "list");
    }
}
