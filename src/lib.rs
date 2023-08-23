pub mod algebra;
pub mod domain;
pub mod extension;

pub mod prelude {
    use std::path::{Path, PathBuf};
    use std::str::FromStr;

    use once_cell::sync::Lazy;

    #[doc(inline)]
    pub use crate::algebra::prelude::*;
    #[doc(inline)]
    pub use crate::domain::prelude::*;
    #[doc(inline)]
    pub use crate::extension::prelude::*;

    pub static WORKDIR: Lazy<PathBuf> = Lazy::new(|| {
        let home_env_var = {
            #[cfg(not(target_os = "windows"))]
            {
                env!("HOME")
            }
            #[cfg(target_os = "windows")]
            {
                env!("USERPROFILE")
            }
        };

        let path = Path::new(&home_env_var).join(".config").join("workflows");
        let configuration: Result<Result<Configuration, Error>, Error> =
            std::fs::read_to_string(path)
                .map(|s| Configuration::from_str(&s))
                .map_err(|e| {
                    Error::InvalidConfiguration(Some(
                        format!("Failed to read configuration file: {}", e).into(),
                    ))
                });

        let home = std::env::var("WORKFLOW_DIR").map(|s| Path::new(&s).to_path_buf());
        match home {
            Ok(home) => home,
            Err(_) => match configuration {
                Ok(Ok(path)) => Path::new(path.workflow_dir()).to_path_buf(),
                _ => Path::new(home_env_var).join(".workflows"),
            },
        }
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
