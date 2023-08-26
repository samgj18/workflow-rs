use std::path::{Path, PathBuf};

use crate::prelude::{Error, FileExtension};

pub struct FileMetadata {
    path: PathBuf,
    name: String,
}

impl FileMetadata {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    path: PathBuf,
}

impl File {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    pub fn read_dir(&self) -> Result<Vec<FileMetadata>, Error> {
        std::fs::read_dir(self.path.clone())
            .map_err(|e| Error::Io(Some(e.into())))?
            .try_fold::<Vec<FileMetadata>, _, Result<Vec<FileMetadata>, Error>>(
                Vec::new(),
                |mut result, file| {
                    let file = file.map_err(|e| Error::Io(Some(e.into())))?;
                    let path = file.path();
                    if path.is_file() {
                        let name = FileExtension::format(
                            path.file_name()
                                .ok_or(Error::InvalidName(None))?
                                .to_str()
                                .ok_or(Error::InvalidName(None))?,
                        );

                        result.push(FileMetadata { path, name });
                    }

                    Ok(result)
                },
            )
    }

    pub fn remove_all(&self) -> Result<Self, Error> {
        std::fs::remove_dir_all(&self.path).map_err(|e| Error::Io(Some(e.into())))?;
        Ok(self.clone())
    }

    pub fn create_dir_all(&self) -> Result<Self, Error> {
        std::fs::create_dir_all(&self.path).map_err(|e| Error::Io(Some(e.into())))?;
        Ok(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const WORKFLOW: &str = {
        #[cfg(target_os = "windows")]
        {
            ".\\specs\\workflow"
        }
        #[cfg(not(target_os = "windows"))]
        {
            "./specs/workflow"
        }
    };

    #[test]
    fn test_read_dir() {
        let path = Path::new(WORKFLOW).join("test_read_dir");
        let file = File::new(&path);
        let files = file.read_dir().unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[1].name(), "test_read_dir.yml");
        assert_eq!(files[0].name(), "test_read_dir2.yml");
    }

    #[test]
    fn test_create_dir_all() {
        let path = Path::new(WORKFLOW).join("test_create_dir_all");
        let file = File::new(&path);
        file.create_dir_all().unwrap();
        assert_eq!(file.read_dir().unwrap().len(), 0);
    }
}
