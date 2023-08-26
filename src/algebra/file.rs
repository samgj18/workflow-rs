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
