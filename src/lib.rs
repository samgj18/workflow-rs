pub mod domain;
pub mod internal;

pub mod prelude {
    use once_cell::sync::Lazy;

    pub use crate::domain::prelude::*;
    pub use crate::internal::prelude::*;

    pub static WORKDIR: Lazy<String> = Lazy::new(|| {
        let home = std::env::var("HOME").expect("Failed to get home directory");
        format!("{}/.config/workflows", home)
    });

    // This is fine because a CLI application is a blocking application. Hence, we can use a global
    // variable to store the index, writer, and reader.
    pub static INDEX: Lazy<Index> = Lazy::new(|| Index::new().expect("Failed to create index"));
    pub static WRITER: Lazy<Writer> =
        Lazy::new(|| Writer::new(&INDEX).expect("Failed to create writer"));
    pub static READER: Lazy<Reader> =
        Lazy::new(|| Reader::new(&INDEX).expect("Failed to create reader"));

    pub type Unit = ();
}
