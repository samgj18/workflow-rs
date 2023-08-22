use tantivy::schema::Schema as TantivySchema;

use crate::prelude::Workflow;

pub trait Schema {
    /// Create a schema for the index.
    fn schema() -> TantivySchema;
}

impl Schema for Workflow {
    fn schema() -> TantivySchema {
        let mut schema_builder = TantivySchema::builder();
        schema_builder.add_text_field("id", tantivy::schema::TEXT | tantivy::schema::STORED);
        schema_builder.add_json_field("body", tantivy::schema::TEXT | tantivy::schema::STORED);
        schema_builder.build()
    }
}
