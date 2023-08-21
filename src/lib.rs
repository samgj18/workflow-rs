pub mod internal;
pub mod domain;

pub mod prelude {
    use once_cell::sync::Lazy;

    pub use crate::internal::prelude::*;
    pub use crate::domain::prelude::*;

    pub static WORKDIR: Lazy<String> = Lazy::new(|| {
        let home = std::env::var("HOME").expect("Failed to get home directory");
        format!("{}/.config/workflows", home)
    });

    pub type Unit = ();
}
