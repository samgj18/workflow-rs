use std::{
    env,
    fmt::{Debug, Formatter},
    path::Path,
};

use tantivy::{schema::Schema, Index as TantivyIndex};

use crate::{
    domain::prelude::Error,
    internal::schema::Schema as SchemaTrait,
    prelude::{Workflow, INDEX_DIR, WORKDIR},
};

#[derive(Clone)]
pub struct Index {
    index: TantivyIndex,
}

impl Debug for Index {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexAlgebra")
            .field("index", &self.index)
            .field("writer", &"[redacted]")
            .field("reader", &"[redacted]")
            .finish()
    }
}

impl Index {
    pub fn new_at_dir_or_ram<T>(schema: &Schema, path: Option<T>) -> Result<Self, Error>
    where
        T: Sized + AsRef<Path>,
    {
        let index = if let Some(path) = path {
            TantivyIndex::open_in_dir(&path).or_else(|e| -> Result<TantivyIndex, Error> {
                match e {
                    tantivy::TantivyError::OpenDirectoryError(_)
                    | tantivy::TantivyError::OpenReadError(_) => {
                        std::fs::create_dir_all(&path).map_err(|e| {
                            Error::SchemaError(Some(
                                format!("Error creating index dir: {:?}", e).into(),
                            ))
                        })?;
                        TantivyIndex::create_in_dir(path, schema.clone()).map_err(|e| {
                            Error::SchemaError(Some(format!("Error opening index: {:?}", e).into()))
                        })
                    }
                    tantivy::TantivyError::LockFailure(e, desc) => Err(Error::SchemaError(Some(
                        format!("Error locking index: {:?} ({:?})", e, desc).into(),
                    ))),
                    _ => Err(Error::SchemaError(Some(
                        format!("Error opening index: {:?}", e).into(),
                    ))),
                }
            })?
        } else {
            TantivyIndex::create_in_ram(schema.clone())
        };

        Ok(Index { index })
    }

    pub fn new() -> Result<Self, Error> {
        let current_dir = env::current_dir().map_err(|e| {
            Error::SchemaError(Some(format!("Error getting current dir: {:?}", e).into()))
        })?;

        // TODO: Make this configurable. Bad for testing. See `test_execute_scan`.
        let index_path = Path::new(&current_dir)
            .join::<&str>(&WORKDIR)
            .join::<&str>(INDEX_DIR);

        let schema = Workflow::schema();

        let index = Index::new_at_dir_or_ram(&schema, Some(index_path))?;

        Ok(index)
    }

    pub fn inner(&self) -> TantivyIndex {
        self.index.clone()
    }

    pub fn schema(&self) -> Schema {
        self.index.schema()
    }
}
