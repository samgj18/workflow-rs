use std::{
    collections::HashSet,
    fs::ReadDir,
    io::{Seek, SeekFrom},
};

use crate::prelude::{prepare_workflows, Error, File, Unit, Workflow, Writer};

pub const HISTORY: &str = "history.json";
pub const INDEX_DIR: &str = "index";

pub struct Crawler {}

#[derive(Debug)]
struct Finding {
    visited: HashSet<String>,
    non_visited: HashSet<String>,
}

impl Finding {
    pub fn union(&self) -> HashSet<String> {
        self.visited.union(&self.non_visited).cloned().collect()
    }
}

// TODO: Refactor this.
impl Crawler {
    /// Visit the given directory, look for history.json if it exists, create it if it doesn't.
    /// `history.json` is a file that contains the names of all the files that have been visited and indexed.
    /// If the file exists, it will be read and the names will be used to skip files that have already been indexed.
    /// If the file doesn't exist, it will be created and the names of the files that have been indexed will be written to it.
    ///
    /// # Arguments
    ///
    /// * `directory` - The directory to visit.
    ///
    /// # Returns
    /// A `Result` containing a `Finding` if the directory was visited successfully, or an `Error` if it wasn't.
    fn finding(names: HashSet<String>, directory: &str) -> Result<Finding, Error> {
        let mut file = std::fs::OpenOptions::new();
        let path = format!("{}/{}", directory, HISTORY);
        let exists = File::new(&path).exists();

        if !exists {
            Ok(Finding {
                visited: HashSet::new(),
                non_visited: names,
            })
        } else {
            let mut writer: std::fs::File = file
                .read(true)
                .write(true)
                .open(path)
                .map_err(|e| Error::Io(Some(e.into())))?;

            let history: HashSet<String> = serde_json::from_reader(&mut writer)
                .map_err(|e| Error::ParseError(Some(e.into())))?;

            Ok(Finding {
                non_visited: names.difference(&history).cloned().collect(),
                visited: history,
            })
        }
    }

    /// Write the given set to the given writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - The writer to write to.
    /// * `set` - The set to write.
    fn write(directory: &str, set: &HashSet<String>) -> Result<Unit, Error> {
        let mut writer = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(directory)
            .map_err(|e| Error::Io(Some(e.into())))?;

        writer
            .seek(SeekFrom::Start(0))
            .map_err(|e| Error::Io(Some(e.into())))?;
        writer.set_len(0).map_err(|e| Error::Io(Some(e.into())))?;

        serde_json::to_writer(&mut writer, &set).map_err(|e| Error::ParseError(Some(e.into())))?;
        Ok(())
    }

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
    /// * `writer` - The writer to use for indexing.
    pub fn crawl(directory: &str, writer: &Writer) -> Result<Unit, Error> {
        let path = format!("{}/{}", directory, INDEX_DIR);
        let files: ReadDir = std::fs::read_dir(directory).map_err(|e| Error::Io(Some(e.into())))?;

        let names: Vec<String> = files
            .filter(|file| {
                file.as_ref().map_or(false, |file| {
                    file.metadata().map_or(false, |metadata| metadata.is_file())
                })
            })
            .try_fold::<Vec<String>, _, Result<Vec<String>, Error>>(
                Vec::new(),
                |mut names, file| {
                    let file = file.map_err(|e| Error::Io(Some(e.into())))?;
                    let path = file.path();
                    let name = path
                        .file_name()
                        .ok_or(Error::InvalidName(None))?
                        .to_str()
                        .ok_or(Error::InvalidName(None))?;

                    names.push(name.to_owned());
                    Ok(names)
                },
            )?;

        let finding = Self::finding(names.into_iter().collect(), path.as_str())?;
        let path = format!("{}/{}/{}", directory, INDEX_DIR, HISTORY);
        Self::write(&path, &finding.union())?;

        let names: &[&str] = &finding
            .non_visited
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<&str>>();

        let workflows: Vec<(String, Workflow)> = prepare_workflows(names, directory)
            .map_err(|e| Error::SchemaError(Some(e.into())))?
            .into_iter()
            .map(|workflow| (workflow.name().inner().to_owned(), workflow))
            .collect::<Vec<(String, Workflow)>>();

        let workflows: &[(&str, Workflow)] = &workflows
            .iter()
            .map(|(name, workflow)| (name.as_str(), workflow.clone()))
            .collect::<Vec<(&str, Workflow)>>();

        let mut writer = writer.clone();

        writer.add_many(workflows)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const WORKDIR: &str = "./specs";
    #[test]
    fn test_non_visited_when_history_doesnt_exist() {
        let path = format!("{}/{}", WORKDIR, "empty");

        let names = vec!["echo".to_owned()].into_iter().collect();

        let crawled = Crawler::finding(names, &path).unwrap();

        assert_eq!(crawled.non_visited.len(), 1);
        assert!(crawled.visited.is_empty());
        assert!(crawled.non_visited.contains("echo"));
    }

    #[test]
    fn test_non_visited_when_history_exists() {
        let path = format!("{}/{}", WORKDIR, INDEX_DIR);
        let names = vec!["echo".to_owned()].into_iter().collect();

        let crawled = Crawler::finding(names, &path).unwrap();

        assert_eq!(crawled.non_visited.len(), 0);
        assert_eq!(crawled.visited.len(), 1);
    }
}
