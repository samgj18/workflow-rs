use std::path::Path;

use crate::prelude::{Error, Unit};

/// File extension enum for yaml and yml
#[derive(Debug, PartialEq)]
pub enum FileExtension {
    Yaml,
    Yml,
    None,
}

impl<'a> From<&'a str> for FileExtension {
    fn from(value: &'a str) -> Self {
        if value.contains(".yml") {
            FileExtension::Yml
        } else if value.contains(".yaml") {
            FileExtension::Yaml
        } else {
            FileExtension::None
        }
    }
}

pub struct File {
    path: String,
}

impl File {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn exists(&self) -> bool {
        Path::new(&self.path).exists()
    }

    pub fn remove_all(&self) -> Result<Unit, Error> {
        std::fs::remove_dir_all(&self.path).map_err(|e| Error::Io(Some(e.into())))
    }

    pub fn create_dir_all(&self) -> Result<Unit, Error> {
        std::fs::create_dir_all(&self.path).map_err(|e| Error::Io(Some(e.into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_from() {
        let yaml = "test.yaml";
        let yml = "test.yml";
        let none = "test";

        assert_eq!(FileExtension::from(yaml), FileExtension::Yaml);
        assert_eq!(FileExtension::from(yml), FileExtension::Yml);
        assert_eq!(FileExtension::from(none), FileExtension::None);
    }
}
