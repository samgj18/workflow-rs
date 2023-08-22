pub mod args;
pub mod executor;
pub mod parser;
pub mod schema;

pub mod prelude {
    #[doc(inline)]
    pub use super::args::*;
    #[doc(inline)]
    pub use super::executor::*;
    #[doc(inline)]
    pub use super::parser::*;
    #[doc(inline)]
    pub use super::schema::*;

    use crate::prelude::{Error, FileExtension, Workflow};
    use std::collections::HashSet;

    /// Load the workflow file from the given location.
    pub(crate) fn load_workflow_file(workdir: &str, value: &str) -> Result<String, Error> {
        let path = format!("{}/{}", workdir, value);
        std::fs::read_to_string(path).map_err(|e| Error::ReadError(Some(e.into())))
    }

    /// Parse the workflow string into a workflow.
    pub(crate) fn parse_workflow_string(workflow: String) -> Result<Workflow, Error> {
        serde_yaml::from_str::<Workflow>(&workflow).map_err(|e| Error::ParseError(Some(e.into())))
    }

    /// Prepare the workflow for execution.
    pub fn prepare_workflows(names: &[&str], location: &str) -> Result<Vec<Workflow>, Error> {
        let values = names
            .iter()
            .flat_map(|name| FileExtension::format(name))
            .collect::<HashSet<String>>();

        values
            .iter()
            .map(|value| load_workflow_file(location, value).and_then(parse_workflow_string))
            .collect::<Result<Vec<Workflow>, Error>>()
    }
}
