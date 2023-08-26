pub mod configuration;
pub mod crawler;
pub mod file;
pub mod hasher;

pub mod prelude {
    #[doc(inline)]
    pub use super::configuration::*;
    #[doc(inline)]
    pub use super::crawler::*;
    #[doc(inline)]
    pub use super::file::*;
    #[doc(inline)]
    pub use super::hasher::*;
}
