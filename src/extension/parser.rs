use std::collections::HashMap;

use inquire::{required, Text};

use crate::{
    domain::{
        args::{Argument, ArgumentDefault},
        error::Error,
        workflow::Workflow,
    },
    prelude::Unit,
};

pub trait Parser {
    type Error;
    type Output;
    type Args;
    /// Parse the input string into a type U or an `Error`
    ///
    /// # Arguments
    ///
    /// * `input` - An `Option` of type `I`
    ///
    /// # Returns
    ///
    /// * A `Result` with a type `T` or an `Error`
    fn try_parse(&self, input: Self::Args) -> Result<Self::Output, Self::Error>;
}

pub type Precedence = HashMap<String, String>;

impl Parser for Workflow {
    type Error = Error;
    type Output = Precedence;
    type Args = Unit;

    fn try_parse(&self, _: Self::Args) -> Result<Self::Output, Self::Error> {
        let precedence = self.arguments().iter().try_fold(
            HashMap::new(),
            |mut acc, argument| -> Result<HashMap<String, String>, Error> {
                let name = argument.name().inner().to_string();
                let suggester = self.clone();
                let value = if !argument.values().is_empty() {
                    Text::new(argument.name().inner())
                        .with_validator(required!("This field is required"))
                        .with_help_message(argument.def_description())
                        .with_autocomplete(move |i: &str| suggester.suggestion(i, name.as_str()))
                        .prompt()
                        .map_err(|e| Error::ReadError(Some(e.into())))?
                } else {
                    Text::new(argument.name().inner())
                        .with_help_message(argument.def_description())
                        .prompt()
                        .map_err(|e| Error::ReadError(Some(e.into())))?
                };

                if !value.is_empty() {
                    acc.insert(argument.name().inner().to_string(), value);
                }
                Ok(acc)
            },
        )?;

        let mut arguments = HashMap::new();
        self.arguments().iter().for_each(|arg| {
            if let Ok(Some(args)) = arg.try_parse(Some(precedence.clone())) {
                arguments.extend(args);
            }
        });

        Ok(arguments)
    }
}

impl Parser for Argument {
    type Error = Error;
    type Output = Option<Precedence>;
    type Args = Option<HashMap<String, String>>;

    fn try_parse(&self, precedence: Self::Args) -> Result<Self::Output, Self::Error> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::args::Argument;

    #[test]
    fn test_parse_argument() {
        let argument = Argument::slim("test_arg", None, vec![]);
        let argument = argument.try_parse(None).unwrap().unwrap();

        assert_eq!(argument.len(), 1);
        assert_eq!(
            argument.get("test_arg"),
            Some(&"<insert value>".to_string())
        );
    }

    #[test]
    fn test_parse_argument_with_precedence() {
        let argument = Argument::slim("test_arg", None, vec![]);

        let mut precedence = HashMap::new();
        precedence.insert("test_arg".into(), "test".into());

        let argument = argument.try_parse(Some(precedence)).unwrap().unwrap();

        assert_eq!(argument.len(), 1);
        assert_eq!(argument.get("test_arg"), Some(&"test".to_string()));
    }

    #[test]
    fn test_parse_argument_without_precedence_and_default() {
        let argument = Argument::slim("test_arg", Some("super test"), vec![]);

        let argument = argument.try_parse(None).unwrap().unwrap();

        assert_eq!(argument.len(), 1);
        assert_eq!(argument.get("test_arg"), Some(&"super test".to_string()));
    }

    // Depends on https://github.com/mikaelmello/inquire/issues/70
    // #[test]
    // fn test_parse_workflow() {
    // }

    // Depends on https://github.com/mikaelmello/inquire/issues/70
    // #[test]
    // fn test_parse_workflow_with_precedence() {
    // }

    // Depends on https://github.com/mikaelmello/inquire/issues/70
    // #[test]
    // fn test_parse_workflow_without_precedence_and_default() {
    // }
}
