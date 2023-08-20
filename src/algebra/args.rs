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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::Run;

    pub const WORKDIR: &str = "./specs";

    #[test]
    fn test_load_workflow_file() {
        let value = "echo.yml";
        let result = load_workflow_file(WORKDIR, value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_workflow_string() {
        let workflow = r#"
            name: test
            command: test
        "#;
        let result = parse_workflow_string(workflow.to_owned());
        assert!(result.is_ok());
    }

    #[test]
    fn test_prepare() {
        let command = Command::Run(Run::new("echo.yml", Some(WORKDIR)));
        let result = command.prepare();

        let name = result.as_ref().unwrap().as_ref().unwrap().name().inner();
        let description = result
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .description()
            .unwrap()
            .inner();
        let command = result.as_ref().unwrap().as_ref().unwrap().command().inner();
        let is_some = result.is_ok() && result.as_ref().unwrap().is_some();

        assert!(is_some);
        assert_eq!(name, "echo");
        assert_eq!(description, "Echo a message with a list of arguments");
        assert_eq!(command, "echo \"This is a cool echo to try out: {{sshKeyPath}} and User: {{userName}} <{{userEmail}}>\"");
    }
}
