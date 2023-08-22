pub mod algebra;
pub mod domain;
pub mod extension;

pub mod prelude {
    use once_cell::sync::Lazy;

    #[doc(inline)]
    pub use crate::algebra::prelude::*;
    #[doc(inline)]
    pub use crate::domain::prelude::*;
    #[doc(inline)]
    pub use crate::extension::prelude::*;

    pub static WORKDIR: Lazy<String> = Lazy::new(|| {
        // TODO: Make this a writable file such that user can create a .config/workflows directory
        // and we read the path from there.
        let home = std::env::var("WORKFLOW_DIR");
        let default = format!("{}/.workflows", env!("HOME"));

        home.unwrap_or(default)
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
