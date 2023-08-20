use std::collections::HashSet;

use crate::{
    domain::{
        command::Command,
        prelude::{Error, FileExtension, Workflow},
    },
    prelude::WORKDIR,
};

pub trait Prepare {
    /// Given a workflow name, returns an `Option` of `Workflow`
    /// depending on whether the command actually needs a workflow or not
    /// or an `Error`.
    ///
    /// If the command does not need a workflow, then it returns `None`.
    ///
    /// # Arguments
    /// * `name` - A `&str` that represents the workflow name
    ///
    /// # Returns
    /// * A `Workflow` struct or an `Error`
    fn prepare(&self) -> Result<Option<Workflow>, Error>;
}

impl Prepare for Command {
    fn prepare(&self) -> Result<Option<Workflow>, Error> {
        match self {
            Command::Run(command) => {
                let name = command.name();
                let mut values = HashSet::new();
                let location = command.location().unwrap_or(&WORKDIR);

                match FileExtension::from(name) {
                    FileExtension::Yaml | FileExtension::Yml => values.insert(name.to_string()),
                    FileExtension::None => values.insert(format!("{}.yaml", name)),
                };

                values
                    .iter()
                    .map(|value| {
                        load_workflow_file(location, value).and_then(parse_workflow_string)
                    })
                    .collect::<Result<Vec<Workflow>, Error>>()?
                    .pop()
                    .ok_or(Error::InvalidName(None))
                    .map(Some)
            }
            Command::List(_) => Ok(None),
        }
    }
}

fn load_workflow_file(workdir: &str, value: &str) -> Result<String, Error> {
    let path = format!("{}/{}", workdir, value);
    std::fs::read_to_string(path).map_err(|e| Error::ReadError(Some(e.into())))
}

fn parse_workflow_string(workflow: String) -> Result<Workflow, Error> {
    serde_yaml::from_str::<Workflow>(&workflow).map_err(|e| Error::ParseError(Some(e.into())))
}
