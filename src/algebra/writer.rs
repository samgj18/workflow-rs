use std::sync::{Arc, Mutex};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map, Value};
use tantivy::{schema::Field as TantivyField, Document, IndexWriter, Opstamp};

use crate::prelude::*;

#[derive(Clone)]
pub struct Writer {
    writer: Arc<Mutex<IndexWriter>>,
    index: Index,
}

impl Writer {
    pub fn new(index: &Index) -> Result<Self, Error> {
        let writer = index
            .inner()
            .writer(50_000_000)
            .map_err(|e| {
                Error::SchemaError(Some(format!("Error creating index writer: {:?}", e).into()))
            })
            .map(|w| Arc::new(Mutex::new(w)))?;

        Ok(Self {
            writer,
            index: index.clone(),
        })
    }

    pub fn inner(self) -> Arc<Mutex<IndexWriter>> {
        self.writer
    }

    /// Add a document to the index.
    /// Returns the opstamp of the document added.
    ///
    /// # Note
    /// This method commits on every call. If you want to add many documents at once, use `add_many`.
    pub fn add<T>(&mut self, id: &str, body: &T) -> Result<Opstamp, Error>
    where
        T: Serialize + DeserializeOwned,
    {
        let body = serde_json::to_value(body).map_err(|e| {
            Error::SchemaError(Some(format!("Error serializing document: {:?}", e).into()))
        })?;

        let id_field = self.field("id")?;
        let body_field = self.field("body")?;

        let json = serde_json::to_value(body).map_err(|e| {
            Error::SchemaError(Some(format!("Error serializing document: {:?}", e).into()))
        })?;

        let body: Map<String, Value> = serde_json::from_value(json).map_err(|e| {
            Error::SchemaError(Some(
                format!("Error deserializing document: {:?}", e).into(),
            ))
        })?;

        let mut doc = Document::default();
        doc.add_text(id_field, id);
        doc.add_json_object(body_field, body);

        self.add_document(doc)
    }

    /// Returns a field from the schema or an error if the field does not exist.
    pub fn field(&self, name: &str) -> Result<TantivyField, Error> {
        self.index
            .schema()
            .get_field(name)
            .map_err(|e| Error::SchemaError(Some(format!("Error getting field: {:?}", e).into())))
    }

    /// Add many documents to the index.
    /// Returns a vector of opstamps for each document added. The last `opstamp` is the commit opstamp.
    ///
    /// This a lower level method that allows you to add many documents at once. Use `add_many` when possible.
    ///
    /// # Note
    /// Commit returns the `opstamp` of the last document that made it in the commit.
    pub fn add_many<T>(&mut self, documents: &[(&str, T)]) -> Result<Vec<Opstamp>, Error>
    where
        T: Serialize + DeserializeOwned,
    {
        let id_field = self.field("id")?;
        let body_field = self.field("body")?;

        let documents: Result<Vec<Document>, Error> =
            documents
                .iter()
                .try_fold(Vec::new(), |mut acc, (id, body)| {
                    let json = serde_json::to_value(body).map_err(|e| {
                        Error::SchemaError(Some(
                            format!("Error serializing document: {:?}", e).into(),
                        ))
                    })?;

                    let body: Map<String, Value> = serde_json::from_value(json).map_err(|e| {
                        Error::SchemaError(Some(
                            format!("Error deserializing document: {:?}", e).into(),
                        ))
                    })?;

                    let mut doc = Document::default();
                    doc.add_text(id_field, *id);
                    doc.add_json_object(body_field, body);
                    acc.push(doc);

                    Ok(acc)
                });

        let documents = documents?;

        self.add_many_documents(documents)
    }

    /// Add a document to the index.
    /// Returns the opstamp of the document added.
    ///
    /// This a lower level method that allows you to add a document directly. Use `add` when possible.
    ///
    /// # Note
    /// This method commits on every call. If you want to add many documents at once, use `add_many`.
    pub fn add_document(&mut self, document: Document) -> Result<Opstamp, Error> {
        self.add_many_documents(vec![document])
            .and_then(|mut opstamps| {
                opstamps
                    .pop()
                    .ok_or_else(|| Error::SchemaError(Some("No opstamp returned".into())))
            })
    }

    /// Add many documents to the index.
    /// Returns a vector of opstamps for each document added. The last `opstamp` is the commit opstamp.
    ///
    /// # Note
    /// Commit returns the `opstamp` of the last document that made it in the commit.
    pub fn add_many_documents(&mut self, documents: Vec<Document>) -> Result<Vec<Opstamp>, Error> {
        let writer = &mut self.writer.lock().map_err(|e| {
            Error::SchemaError(Some(format!("Error locking index writer: {:?}", e).into()))
        })?;
        let opstamps: Result<Vec<u64>, Error> = documents
            .into_iter()
            .map(|document| {
                writer.add_document(document).map_err(|e| {
                    Error::SchemaError(Some(
                        format!("Error adding document to index: {:?}", e).into(),
                    ))
                })
            })
            .collect();

        let last_opstamp = writer.commit().map_err(|e| {
            Error::SchemaError(Some(format!("Error committing index: {:?}", e).into()))
        })?;

        opstamps.map(|mut opstamps| {
            opstamps.push(last_opstamp);
            opstamps
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::algebra::prelude::{Index, Reader};
    use crate::domain::prelude::SearchTerm;

    use fake::faker::lorem::en::*;
    use fake::{Fake, Faker};
    use serde::{Deserialize, Serialize};
    use tantivy::{doc, schema::*};

    use super::Writer;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct TestId(pub String);

    fn string() -> String {
        Faker.fake::<String>()
    }

    fn lorem() -> String {
        Words(10..20).fake::<Vec<String>>().join(" ")
    }

    fn index() -> Index {
        let path = {
            #[cfg(target_os = "windows")]
            {
                format!(".\\specs\\data_{}", string())
            }
            #[cfg(not(target_os = "windows"))]
            {
                format!("./specs/data_{}", string())
            }
        };

        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("id", TEXT | STORED);
        schema_builder.add_json_field("body", TEXT | STORED);

        let schema = schema_builder.build();
        let path = PathBuf::from(path);

        Index::new_at_dir_or_ram::<PathBuf>(&schema, Some(path)).unwrap()
    }

    fn writer(index: Index) -> Writer {
        Writer::new(&index).unwrap()
    }

    fn reader(index: Index) -> Reader {
        Reader::new(&index).unwrap()
    }

    fn schema(index: Index) -> Schema {
        index.inner().schema()
    }

    #[test]
    fn test_create_index() {
        let schema = index().inner().schema();
        let fields = schema.fields().collect::<Vec<_>>();

        assert_eq!(schema.fields().count(), 2);
        assert_eq!(fields.first().unwrap().0.field_id(), 0);
        assert_eq!(fields.first().unwrap().1.name(), "id");
        assert!(fields.first().unwrap().1.field_type().is_indexed());
    }

    #[test]
    fn test_add_document() {
        let index = index();
        let mut writer = writer(index.clone());
        let schema = schema(index.clone());
        let id = schema.get_field("id").unwrap();
        let body = schema.get_field("body").unwrap();
        let lorem = lorem();

        let doc = doc!(
            id => "The Old Man and the Sea",
            body => serde_json::json!({ "body": lorem }),
        );

        writer.add_document(doc).unwrap();

        // wait for the index to be written
        std::thread::sleep(std::time::Duration::from_millis(2000));

        let res = reader(index)
            .get(&SearchTerm::from("sea whale"), &["id", "body"], None)
            .unwrap();

        let opstamp = res.first().unwrap().0;
        let value = &res.first().unwrap().1;

        let expected = serde_json::json!({
            "body": [{
                "body": lorem
            }],
            "id": ["The Old Man and the Sea"],
        });

        assert_eq!(opstamp, 0.28768212_f32);
        assert_eq!(value, &expected);
    }

    #[test]
    fn test_add_many_documents() {
        let index = index();
        let mut writer = writer(index.clone());
        let reader = reader(index.clone());
        let schema = schema(index);
        let mut document = Document::default();
        document.add_text(schema.get_field("id").unwrap(), "1");
        document.add_text(schema.get_field("id").unwrap(), "2");

        writer.add_many_documents(vec![document]).unwrap();

        // wait for the index to be written
        std::thread::sleep(std::time::Duration::from_millis(2000));

        let id = schema.get_field("id").unwrap();

        let all = reader.get_all_raw().unwrap();

        let ids = all.first().unwrap().get_all(id).collect::<Vec<_>>();

        assert_eq!(all.len(), 1);
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0].as_text().unwrap(), "1");
        assert_eq!(ids[1].as_text().unwrap(), "2");
    }
}
