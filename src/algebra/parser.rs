use std::collections::HashMap;

use inquire::Text;

use crate::domain::{
    args::{Argument, ArgumentDefault},
    error::Error,
    workflow::Workflow,
};

pub trait Parser<T, I> {
    /// Parse the input string into a type U or an `Error`
    ///
    /// # Arguments
    ///
    /// * `input` - An `Option` of type `I`
    ///
    /// # Returns
    ///
    /// * A `Result` with a type `T` or an `Error`
    fn try_parse<U>(&self, input: Option<I>) -> Result<T, U>
    where
        U: From<Error>;
}

pub type Precedence = HashMap<String, String>;

impl Parser<Precedence, String> for Workflow {
    fn try_parse<U>(&self, _: Option<String>) -> Result<Precedence, U>
    where
        U: From<Error>,
    {
        let precedence = self.arguments().iter().try_fold(
            HashMap::new(),
            |mut acc, argument| -> Result<HashMap<String, String>, Error> {
                let value = Text::new(argument.name().inner())
                    .prompt()
                    .map_err(|e| Error::ReadError(Some(e.into())))?;

                if !value.is_empty() {
                    acc.insert(argument.name().inner().to_string(), value);
                }
                Ok(acc)
            },
        )?;

        let mut arguments = HashMap::new();
        self.arguments().iter().for_each(|arg| {
            if let Ok(Some(args)) = arg.try_parse::<Error>(Some(precedence.clone())) {
                arguments.extend(args);
            }
        });

        Ok(arguments)
    }
}

impl Parser<Option<Precedence>, HashMap<String, String>> for Argument {
    fn try_parse<U>(
        &self,
        precedence: Option<HashMap<String, String>>,
    ) -> Result<Option<Precedence>, U>
    where
        U: From<Error>,
    {
        let mut arguments = HashMap::new();
        let default_value = self
            .default()
            .unwrap_or(&ArgumentDefault::new("<insert value>".into()))
            .inner()
            .to_string();

        let name = self.name().inner();
        arguments.insert(name.to_owned(), default_value);

        if let Some(value) = precedence.and_then(|p| p.get(name).cloned()) {
            arguments.insert(name.to_owned(), value);
        }

        match arguments {
            arguments if arguments.is_empty() => Ok(None),
            arguments => Ok(Some(arguments)),
        }
    }
}
