/// File extension enum for yaml and yml
pub enum FileExtension {
    Yaml,
    Yml,
    None,
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
