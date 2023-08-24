use std::path::Path;

use crate::prelude::{prepare_workflows, Error, FileExtension, Store, Unit, WorkStore};

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
        let mut store = store.clone();
        std::fs::read_dir(directory)
            .map_err(|e| Error::Io(Some(e.into())))?
            .try_for_each::<_, Result<Unit, Error>>(|file| {
                let file = file.map_err(|e| Error::Io(Some(e.into())))?;
                let path = file.path();
                if path.is_file() {
                    let name = FileExtension::format(
                        path.file_name()
                            .ok_or(Error::InvalidName(None))?
                            .to_str()
                            .ok_or(Error::InvalidName(None))?,
                    );

                    let workflow = prepare_workflows(&[name.as_str()], directory)?
                        .into_iter()
                        .next()
                        .ok_or(Error::SchemaError(None))?;

                    store.insert(name.as_str(), workflow)?;
                }

                Ok(())
            })
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
    fn test_crawl() {
        set_env_var();
        Crawler::crawl(Path::new(WORKFLOW), &STORE).unwrap();

        let workflows = STORE.search().unwrap();

        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].name().inner(), "echo");
    }
}
