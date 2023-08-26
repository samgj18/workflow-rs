use crate::prelude::*;

pub trait Prepare {
    /// The type of the output.
    type Output;
    /// The type of the error.
    type Error;

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
    fn prepare(&self) -> Result<Self::Output, Self::Error>;
}

impl Prepare for Run {
    type Output = Workflow;
    type Error = Error;

    fn prepare(&self) -> Result<Workflow, Error> {
        let id = self.name().trim().to_lowercase().replace(['-', ' '], "_");

        STORE.get(&id)?.ok_or(Error::InvalidName(None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::Run;

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
    fn test_prepare_run() {
        set_env_var();

        let workflow = Workflow::new("echo", "echo \"This is a cool echo to try out: {{sshKeyPath}} and User: {{userName}} <{{userEmail}}>\"", Vec::new());
        STORE.clone().insert_all(vec![workflow]).unwrap();

        let command = Run::new("echo.yml");
        let result = command.prepare();

        let binding = result.as_ref().unwrap().clone().id();
        let id = binding.inner();
        let command = result.as_ref().unwrap().command().inner();

        assert_eq!(id, "echo");
        assert_eq!(command, "echo \"This is a cool echo to try out: {{sshKeyPath}} and User: {{userName}} <{{userEmail}}>\"");
    }
}
