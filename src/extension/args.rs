use crate::prelude::*;

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
                let names: &[&str] = &[command.name()];

                prepare_workflows(names, &WORKDIR)?
                    .pop()
                    .ok_or(Error::InvalidName(None))
                    .map(Some)
            }
            Command::List(_) => Ok(None),
            Command::Search(_) => Ok(None),
            Command::Index(_) => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::Run;
    use std::path::Path;

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
    fn test_load_workflow_file() {
        let value = "echo.yml";
        let result = load_workflow_file(Path::new(WORKFLOW), Path::new(value));
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
        set_env_var();

        let command = Command::Run(Run::new("echo.yml"));
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
