pub mod configuration;
pub mod crawler;
pub mod index;
pub mod reader;
pub mod writer;

pub mod prelude {
    #[doc(inline)]
    pub use super::configuration::*;
    #[doc(inline)]
    pub use super::crawler::*;
    #[doc(inline)]
    pub use super::index::*;
    #[doc(inline)]
    pub use super::reader::*;
    #[doc(inline)]
    pub use super::writer::*;
}
