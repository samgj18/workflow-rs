#[derive(Debug, PartialEq)]
pub struct Output {
    r#type: String,
    message: String,
}

impl Output {
    pub fn new(r#type: &str, message: &str) -> Self {
        Self {
            r#type: r#type.to_string(),
            message: message.to_string(),
        }
    }

    pub fn r#type(&self) -> &str {
        &self.r#type
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}
