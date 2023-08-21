use std::{
    collections::HashSet,
    fs::ReadDir,
    io::{Seek, SeekFrom},
};

use crate::prelude::{prepare_workflows, Error, Unit, Workflow, Writer};

const HISTORY: &str = "history.json";

pub struct Crawler {}

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
    /// A set of names of the files that have not been visited.
    fn non_visited(names: HashSet<String>, directory: &str) -> Result<HashSet<String>, Error> {
        let mut file = std::fs::OpenOptions::new();
        let exists = file
            .write(true)
            .read(true)
            .open(format!("{}/{}", directory, HISTORY))
            .is_ok();

        if !exists {
            let writer = file
                .write(true)
                .read(true)
                .create(true)
                .open(format!("{}/{}", directory, HISTORY))
                .map_err(|e| Error::Io(Some(e.into())))?;

            // Write an empty set to the file.
            let empty: HashSet<String> = HashSet::new();
            let merged: HashSet<String> = empty.union(&names).cloned().collect();
            serde_json::to_writer(&writer, &merged)
                .map_err(|e| Error::ParseError(Some(e.into())))?;

            Ok(names)
        } else {
            let mut writer = file
                .read(true)
                .write(true)
                .create(true)
                .open(format!("{}/{}", directory, HISTORY))
                .map_err(|e| Error::Io(Some(e.into())))?;

            let history: HashSet<String> = serde_json::from_reader(&mut writer)
                .map_err(|e| Error::ParseError(Some(e.into())))?;

            let merged: HashSet<String> = history.union(&names).cloned().collect();

            // Empty the file.
            writer
                .seek(SeekFrom::Start(0))
                .map_err(|e| Error::Io(Some(e.into())))?;
            writer.set_len(0).map_err(|e| Error::Io(Some(e.into())))?;

            serde_json::to_writer(&mut writer, &merged)
                .map_err(|e| Error::ParseError(Some(e.into())))?;

            Ok(names.difference(&history).cloned().collect())
        }
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
    pub fn crawl(directory: &str, writer: &mut Writer) -> Result<Unit, Error> {
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

        let remaining: HashSet<String> = Self::non_visited(
            names.into_iter().collect(),
            format!("{}/{}", directory, "index").as_str(),
        )?;

        let names: &[&str] = &remaining
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

        // Delete empty folder if it exists.
        if std::path::Path::new(&path).exists() {
            std::fs::remove_dir_all(&path).unwrap();
        }

        std::fs::create_dir(&path).unwrap();

        let names = vec!["echo".to_owned()].into_iter().collect();

        let crawled = Crawler::non_visited(names, &path).unwrap();

        assert_eq!(crawled.len(), 1);
        assert!(crawled.contains("echo"));
    }

    #[test]
    fn test_non_visited_when_history_exists() {
        let path = format!("{}/{}", WORKDIR, "index");
        let names = vec!["echo".to_owned()].into_iter().collect();

        let crawled = Crawler::non_visited(names, &path).unwrap();

        assert_eq!(crawled.len(), 0);
    }
}
