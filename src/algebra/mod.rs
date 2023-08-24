pub mod configuration;
pub mod crawler;
pub mod fuzzy;

pub mod prelude {
    #[doc(inline)]
    pub use super::configuration::*;
    #[doc(inline)]
    pub use super::crawler::*;
    #[doc(inline)]
    pub use super::fuzzy::*;
}
