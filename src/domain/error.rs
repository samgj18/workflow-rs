use thiserror::Error as ThisError;
use std::error::Error as StdError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("The workflow name is invalid. {0:?}")]
    InvalidName(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow description is invalid. {0:?}")]
    InvalidDescription(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow command is invalid. {0:?}")]
    InvalidCommand(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow source is invalid. {0:?}")]
    InvalidSource(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow author is invalid. {0:?}")]
    InvalidAuthor(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow version is invalid. {0:?}")]
    InvalidVersion(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow tags are invalid. {0:?}")]
    InvalidTags(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow arguments are invalid. {0:?}")]
    InvalidArguments(#[source] Option<Box<dyn StdError>>),
    #[error("Unable to parse the workflow. {0:?}")]
    ParseError(#[source] Option<Box<dyn StdError>>),
    #[error("Unable to read the workflow. {0:?}")]
    ReadError(#[source] Option<Box<dyn StdError>>),
    #[error("The workflow is invalid. {0:?}")]
    Io(#[from] Option<Box<dyn StdError>>),
}
