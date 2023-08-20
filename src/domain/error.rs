use std::error::Error as StdError;
use std::fmt::{Debug, Formatter, Result};
use thiserror::Error as ThisError;

#[derive(ThisError)]
pub enum Error {
    #[error("The workflow name is invalid.")]
    InvalidName(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow description is invalid.")]
    InvalidDescription(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow command is invalid.")]
    InvalidCommand(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow source is invalid.")]
    InvalidSource(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow author is invalid.")]
    InvalidAuthor(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow version is invalid.")]
    InvalidVersion(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow tags are invalid.")]
    InvalidTags(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow arguments are invalid.")]
    InvalidArguments(#[source] Option<Box<dyn StdError>>),
    #[error("Unable to parse the workflow.")]
    ParseError(#[source] Option<Box<dyn StdError>>),
    #[error("Unable to read the workflow.")]
    ReadError(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow is invalid.")]
    Io(#[from] Option<Box<dyn StdError>>),
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "{}\n", &self)?;

        let mut current = self.source();

        while let Some(cause) = current {
            writeln!(f, "Caused by:\n\t{}", cause)?;
            current = cause.source();
        }

        Ok(())
    }
}
