use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::prelude::{
    Error, File, FileExtension, FileMetadata, Store, Unit, WorkStore, Workflow, WorkflowId,
};

pub struct Crawler {}

impl Crawler {
    /// Crawl the given directory for files and index them.
    ///
    /// It will only index files with the following extensions:
    /// - .yaml
    /// - .yml
    ///
    /// It will ignore all other files or directories.
    ///
    /// # Arguments
    ///
    /// * `directory` - The directory to crawl.
    /// * `store` - The store to insert the data into.
    pub fn crawl(directory: &Path, store: &WorkStore) -> Result<Unit, Error> {
        let mut store: WorkStore = store.clone();
        let files: Vec<FileMetadata> = File::new(directory).read_dir()?;
        let names: &[&str] = &files
            .iter()
            .map(|file| file.name())
            .collect::<HashSet<&str>>()
            .into_iter()
            .collect::<Vec<&str>>();
        let workflows: Vec<Workflow> = prepare_workflows(names, directory)?;

        let workflows_checksums: HashMap<WorkflowId, u64> = workflows
            .iter()
            .map(|workflow| (workflow.id(), workflow.checksum()))
            .collect::<HashMap<WorkflowId, u64>>();

        let stored_workflows = store.get_all()?;

        let stored_workflows_checksums: HashMap<WorkflowId, u64> = stored_workflows
            .iter()
            .map(|workflow| (workflow.id(), workflow.checksum()))
            .collect::<HashMap<WorkflowId, u64>>();

        let is_different = workflows_checksums.iter().any(|(id, checksum)| {
            stored_workflows_checksums
                .get(id)
                .map_or(true, |stored_checksum| stored_checksum != checksum)
        });

        if is_different {
            store.delete_all()?;
            store.insert_all(workflows)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{Store, STORE};

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
    fn test_crawl() {
        set_env_var();
        Crawler::crawl(Path::new(WORKFLOW), &STORE).unwrap();

        let workflows = STORE.get_all().unwrap();

        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].id().inner(), "echo");
    }
}

/// Prepare the workflow for execution.
fn prepare_workflows(names: &[&str], location: &Path) -> Result<Vec<Workflow>, Error> {
    let values = names
        .iter()
        .map(|name| FileExtension::format(name))
        .collect::<HashSet<String>>();

    values
        .iter()
        .map(|value| load_workflow_file(location, Path::new(value)).and_then(parse_workflow_string))
        .collect::<Result<Vec<Workflow>, Error>>()
}

/// Load the workflow file from the given location.
fn load_workflow_file(workdir: &Path, value: &Path) -> Result<String, Error> {
    let path = Path::new(&workdir).join(value);
    std::fs::read_to_string(path).map_err(|e| Error::ReadError(Some(e.into())))
}

/// Parse the workflow string into a workflow.
fn parse_workflow_string(workflow: String) -> Result<Workflow, Error> {
    serde_yaml::from_str::<Workflow>(&workflow).map_err(|e| Error::ParseError(Some(e.into())))
}
