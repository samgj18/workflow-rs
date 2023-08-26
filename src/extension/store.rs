use std::{path::Path, sync::Arc};

use crate::prelude::{Error, Unit, Workflow};

use rocksdb::{Direction, IteratorMode, WriteBatch, DB as RocksDB};

pub trait Store<T> {
    /// Initialize the store with the given file path.
    fn init(file_path: &Path) -> Result<WorkStore, Error>
    where
        Self: Sized;
    /// Insert all the given data into the store.
    /// Be mindful that in this case the key must be present in the data.
    fn insert_all(&mut self, data: Vec<T>) -> Result<Unit, Error>;
    /// Search for the given key in the store starting from the given key.
    fn iterate_from(&self, key: &str) -> Result<Vec<T>, Error>;
    /// Get an entry from the store.
    fn get(&self, key: &str) -> Result<Option<T>, Error>;
    /// Search for all the data in the store starting from the beginning.
    fn get_all(&self) -> Result<Vec<T>, Error>;
    /// Delete the given key from the store.
    fn delete(&mut self, key: &str) -> Result<Unit, Error>;
    /// Delete all the data from the store.
    fn delete_all(&mut self) -> Result<Unit, Error>;
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

// TODO: Abstract common code
impl Store<Workflow> for WorkStore {
    fn init(file_path: &Path) -> Result<WorkStore, Error>
    where
        Self: Sized,
    {
        RocksDB::open_default(file_path)
            .map_err(|e| Error::StoreError(Some(e.into())))
            .map(WorkStore::new)
    }

    fn insert_all(&mut self, data: Vec<Workflow>) -> Result<Unit, Error> {
        let mut db_batch = WriteBatch::default();
        data.into_iter().for_each(|workflow| {
            let v = serde_json::to_string(&workflow).unwrap();
            db_batch.put(workflow.id().inner().as_bytes(), v.as_bytes());
        });

        self.db
            .write(db_batch)
            .map_err(|e| Error::StoreError(Some(e.into())))
    }

    fn iterate_from(&self, key: &str) -> Result<Vec<Workflow>, Error> {
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

    fn get(&self, key: &str) -> Result<Option<Workflow>, Error> {
        self.db
            .get(key.as_bytes())
            .map_err(|e| Error::StoreError(Some(e.into())))
            .and_then(|v| {
                if let Some(v) = v {
                    let workflow = serde_json::from_str::<Workflow>(
                        std::str::from_utf8(&v).unwrap_or_default(),
                    )
                    .map_err(|e| Error::StoreError(Some(e.into())))?;

                    Ok(Some(workflow))
                } else {
                    Ok(None)
                }
            })
    }

    fn get_all(&self) -> Result<Vec<Workflow>, Error> {
        let iterator = self.db.iterator(IteratorMode::End);

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

    fn delete_all(&mut self) -> Result<Unit, Error> {
        let iterator = self.db.iterator(IteratorMode::Start);

        iterator
            .into_iter()
            .try_for_each::<_, Result<Unit, Error>>(|v| {
                if let Ok((k, _)) = v {
                    self.db
                        .delete(k)
                        .map_err(|e| Error::StoreError(Some(e.into())))?;
                }
                Ok(())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::Argument;

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

    #[test]
    fn test_store() {
        let path = Path::new(WORKFLOW).join("test.db");
        std::fs::create_dir(&path).unwrap_or_default();

        let mut store = WorkStore::init(&path).unwrap();

        let name1 = "test";
        let command1 = "test";
        let args1 = vec![Argument::new("test", None, Vec::new())];

        let name2 = "test2";
        let command2 = "test2";
        let args2 = vec![Argument::new("test2", None, Vec::new())];

        let workflow = Workflow::new(name1, command1, args1);
        let workflow2 = Workflow::new(name2, command2, args2);

        store
            .insert_all(vec![workflow.clone(), workflow2.clone()])
            .unwrap();

        let result = store.get(workflow.id().inner()).unwrap();
        let is_some = result.is_some();
        assert!(is_some);
        let workflow1 = result.unwrap();
        assert_eq!(workflow1.name(), workflow.name());
        assert_eq!(workflow1.command(), workflow.command());
        assert_eq!(workflow1.id(), workflow.id());

        let result = store.get(workflow2.id().inner()).unwrap();
        let is_some = result.is_some();
        assert!(is_some);

        let workflow2 = result.unwrap();
        assert_eq!(workflow2.name(), workflow2.name());
        assert_eq!(workflow2.command(), workflow2.command());
        assert_eq!(workflow2.id(), workflow2.id());

        std::fs::remove_dir_all(&path).unwrap_or_default();
    }
}
