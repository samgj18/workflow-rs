use std::collections::HashSet;

/// File extension enum for yaml and yml
#[derive(Debug, PartialEq)]
pub enum FileExtension {
    Yaml,
    Yml,
    None,
}

impl FileExtension {
    pub fn format(name: &str) -> String {
        match FileExtension::from(name) {
            FileExtension::Yaml | FileExtension::Yml => name.to_string(),
            FileExtension::None => format!("{}.yaml", name),
        }
    }

    pub fn format_all(names: HashSet<String>) -> HashSet<String> {
        names
            .iter()
            .map(|name| FileExtension::format(name))
            .collect::<HashSet<String>>()
    }
}

impl<'a> From<&'a str> for FileExtension {
    fn from(value: &'a str) -> Self {
        if value.contains(".yml") {
            FileExtension::Yml
        } else if value.contains(".yaml") {
            FileExtension::Yaml
        } else {
            FileExtension::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_from() {
        let yaml = "test.yaml";
        let yml = "test.yml";
        let none = "test";

        assert_eq!(FileExtension::from(yaml), FileExtension::Yaml);
        assert_eq!(FileExtension::from(yml), FileExtension::Yml);
        assert_eq!(FileExtension::from(none), FileExtension::None);
    }
}
