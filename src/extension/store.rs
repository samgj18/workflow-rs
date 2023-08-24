use std::{path::Path, sync::Arc};

use crate::prelude::{Error, Unit, Workflow};

use rocksdb::{Direction, IteratorMode, DB as RocksDB};

pub trait Store<T> {
    /// Initialize the store with the given file path.
    fn init(file_path: &Path) -> Result<WorkStore, Error>
    where
        Self: Sized;
    /// Insert the given data into the store.
    fn insert(&mut self, key: &str, data: T) -> Result<Unit, Error>;
    /// Search for the given key in the store starting from the given key.
    fn search_from(&self, key: &str) -> Result<Vec<T>, Error>;
    /// Search for all the data in the store starting from the beginning.
    fn search(&self) -> Result<Vec<T>, Error>;
    /// Delete the given key from the store.
    fn delete(&mut self, key: &str) -> Result<Unit, Error>;
}

#[derive(Clone)]
pub struct WorkStore {
    db: Arc<RocksDB>,
}

impl WorkStore {
    pub fn new(db: RocksDB) -> Self {
        Self { db: Arc::new(db) }
    }
}

impl Store<Workflow> for WorkStore {
    fn init(file_path: &Path) -> Result<WorkStore, Error>
    where
        Self: Sized,
    {
        RocksDB::open_default(file_path)
            .map_err(|e| Error::StoreError(Some(e.into())))
            .map(WorkStore::new)
    }

    fn insert(&mut self, key: &str, data: Workflow) -> Result<Unit, Error> {
        let v = serde_json::to_string(&data).map_err(|e| Error::StoreError(Some(e.into())))?;

        self.db
            .put(key.as_bytes(), v.as_bytes())
            .map_err(|e| Error::StoreError(Some(e.into())))
    }

    fn search_from(&self, key: &str) -> Result<Vec<Workflow>, Error> {
        let iterator = self
            .db
            .iterator(IteratorMode::From(key.as_bytes(), Direction::Forward));

        iterator
            .into_iter()
            .try_fold::<Vec<Workflow>, _, Result<Vec<Workflow>, Error>>(
                Vec::new(),
                |mut result, v| {
                    if let Ok((_, v)) = v {
                        let workflow = serde_json::from_str::<Workflow>(
                            std::str::from_utf8(&v).unwrap_or_default(),
                        )
                        .map_err(|e| Error::StoreError(Some(e.into())))?;

                        result.push(workflow);
                    }
                    Ok(result)
                },
            )
    }

    fn delete(&mut self, key: &str) -> Result<Unit, Error> {
        self.db
            .delete(key.as_bytes())
            .map_err(|e| Error::StoreError(Some(e.into())))
    }

    fn search(&self) -> Result<Vec<Workflow>, Error> {
        let iterator = self.db.iterator(IteratorMode::Start);

        iterator
            .into_iter()
            .try_fold::<Vec<Workflow>, _, Result<Vec<Workflow>, Error>>(
                Vec::new(),
                |mut result, v| {
                    if let Ok((_, v)) = v {
                        let workflow = serde_json::from_str::<Workflow>(
                            std::str::from_utf8(&v).unwrap_or_default(),
                        )
                        .map_err(|e| Error::StoreError(Some(e.into())))?;

                        result.push(workflow);
                    }
                    Ok(result)
                },
            )
    }
}
