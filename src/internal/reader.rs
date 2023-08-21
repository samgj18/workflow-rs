use serde_json::Value;
use tantivy::{
    collector::TopDocs,
    query::{AllQuery, FuzzyTermQuery, QueryParser},
    schema::Field as TantivyField,
    DocAddress, Document, Index as TantivyIndex, IndexReader, ReloadPolicy,
    Searcher as TantivySearcher, Term,
};

use crate::prelude::*;

#[derive(Clone)]
pub struct Reader {
    pub reader: IndexReader,
    pub index: Index,
}

impl Reader {
    pub fn new(index: &Index) -> Result<Self, Error> {
        let reader = index
            .inner()
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()
            .map_err(|e| {
                Error::SchemaError(Some(format!("Error creating reader: {:?}", e).into()))
            })?;

        Ok(Self { reader, index: index.clone() })
    }

    pub fn inner(self) -> IndexReader {
        self.reader
    }
    /// Get all raw documents in the index.
    /// This is useful for debugging.
    ///
    /// # Note
    /// This method is very expensive. It should only be used for debugging.
    /// It is not recommended to use this method in production.
    pub fn get_all_raw(&self) -> Result<Vec<Document>, Error> {
        let searcher = self.reader.searcher();
        let docs: Result<Vec<Vec<Document>>, Error> =
            (0..searcher.num_docs()).try_fold(Vec::new(), |mut docs, doc_id| {
                let doc = searcher
                    .search(&AllQuery, &TopDocs::with_limit(100))
                    .map_err(|e| {
                        Error::SchemaError(Some(
                            format!("Error getting document: {:?}, due to {:?}", doc_id, e).into(),
                        ))
                    })?
                    .iter()
                    .map(|(_, doc_address)| {
                        searcher.doc(*doc_address).map_err(|e| {
                            Error::SchemaError(Some(
                                format!("Error getting document: {:?}, due to {:?}", doc_id, e)
                                    .into(),
                            ))
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                docs.push(doc);
                Ok(docs)
            });

        let docs = docs?.into_iter().flatten().collect();

        Ok(docs)
    }

    /// Get a document by term and returns the raw json.
    ///
    /// # Parameters
    /// * `term` - The term to search for. This is anything that could be in the body of any
    /// field in the document.
    /// * `fields` - The fields to search in. See `Field` for more information.
    /// * `limit` - The maximum number of results to return. 100 is the default.
    /// * `query` - When present, this will determine the type of query to use.
    ///
    /// See <https://docs.rs/tantivy/latest/tantivy/query/index.html> for more information.
    /// When `None`, the default `QueryParser` will be used.
    ///
    /// See <https://docs.rs/tantivy/latest/tantivy/query/struct.QueryParser.html> for more information.
    pub fn get(
        &self,
        term: &SearchTerm,
        fields: &[&str],
        limit: Option<&SearchTermLimit>,
    ) -> Result<Vec<(f32, Value)>, Error> {
        self.raw(term.inner(), fields, limit.map(|o| o.inner()).as_ref())
            .map(|results| {
                results
                    .into_iter()
                    .map(|(score, json)| (score, serde_json::from_str::<Value>(&json).unwrap()))
                    .collect()
            })
    }

    /// Get a document by term and tries to deserialize it into the type `A`.
    ///
    /// # Parameters
    /// * `term` - The term to search for. This is anything that could be in the body of any
    /// field in the document.
    /// * `fields` - The fields to search in. See `Field` for more information.
    /// * `limit` - The maximum number of results to return. 100 is the default.
    /// * `query` - When present, this will determine the type of query to use.
    ///
    /// See <https://docs.rs/tantivy/latest/tantivy/query/index.html> for more information.
    /// When `None`, the default `QueryParser` will be used.
    ///
    /// See <https://docs.rs/tantivy/latest/tantivy/query/struct.QueryParser.html> for more information.
    pub fn get_as<A>(
        self,
        term: &SearchTerm,
        fields: &[&str],
        limit: Option<&SearchTermLimit>,
    ) -> Result<Vec<(f32, A)>, Error>
    where
        A: serde::de::DeserializeOwned,
    {
        self.raw(term.inner(), fields, limit.map(|o| o.inner()).as_ref())
            .map(|results| {
                results
                    .into_iter()
                    .map(|(score, json)| {
                        serde_json::from_str::<A>(&json)
                            .map(|doc| (score, doc))
                            .map_err(|e| {
                                Error::SchemaError(Some(
                                    format!("Error deserializing document: {:?}", e).into(),
                                ))
                            })
                    })
                    .collect::<Result<Vec<_>, Error>>()
            })?
    }

    fn raw(
        &self,
        term: &str,
        fields: &[&str],
        limit: Option<&usize>,
    ) -> Result<Vec<(f32, String)>, Error> {
        let index = &self.index;
        let reader = &self.reader; // self.reader()
        let schema = index.schema();

        // Acquiring a searcher is very cheap.
        // You should acquire a searcher every time you start processing a request and
        // release it right after your term is finished.
        let searcher = reader.searcher();

        let tantivy_fields = fields
            .iter()
            .map(|field| {
                schema.get_field(field).map_err(|e| {
                    Error::SchemaError(Some(
                        format!("Error getting field: {:?}, due to {:?}", field, e).into(),
                    ))
                })
            })
            .collect::<Result<Vec<TantivyField>, Error>>()?;

        let top_docs = {
            let exacts = exact(
                term,
                &tantivy_fields,
                &searcher,
                &limit.copied(),
                &index.inner(),
            )?;

            if exacts.is_empty() {
                fuzzy(term, &tantivy_fields, &searcher, &limit.copied())?
            } else {
                exacts
            }
        };

        let results = top_docs
            .into_iter()
            .map(|(score, doc_address)| {
                let retrieved_doc = searcher.doc(doc_address).map_err(|e| {
                    Error::SchemaError(Some(format!("Error getting document: {:?}", e).into()))
                })?;

                let json = schema.to_json(&retrieved_doc);

                Ok((score, json))
            })
            .collect::<Result<_, Error>>()?;

        Ok(results)
    }
}

fn fuzzy(
    term: &str,
    fields: &[TantivyField],
    searcher: &TantivySearcher,
    limit: &Option<usize>,
) -> Result<Vec<(f32, DocAddress)>, Error> {
    Ok(fields
        .iter()
        .flat_map(|field| {
            let term = Term::from_field_text(*field, term);
            let query = FuzzyTermQuery::new(term, 2, true);

            searcher
                .search(&query, &TopDocs::with_limit(limit.unwrap_or(100)))
                .map_err(|e| {
                    Error::SchemaError(Some(format!("Error searching index: {:?}", e).into()))
                })
        })
        .flatten()
        .collect::<Vec<_>>())
}

/// Get a document by term and returns the top documents. This is the default query type.
fn exact(
    term: &str,
    fields: &[TantivyField],
    searcher: &TantivySearcher,
    limit: &Option<usize>,
    index: &TantivyIndex,
) -> Result<Vec<(f32, DocAddress)>, Error> {
    let query_parser = QueryParser::for_index(index, fields.to_vec());

    let term = query_parser
        .parse_query(term)
        .map_err(|e| Error::SchemaError(Some(format!("Error parsing term: {:?}", e).into())))?;

    searcher
        .search(&term, &TopDocs::with_limit(limit.unwrap_or(100)))
        .map_err(|e| Error::SchemaError(Some(format!("Error searching index: {:?}", e).into())))
}
