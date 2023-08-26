pub struct SearchTerm(String);

impl SearchTerm {
    pub fn inner(&self) -> &str {
        &self.0
    }

    pub fn wildcard() -> Self {
        Self("*".to_string())
    }
}

impl From<&str> for SearchTerm {
    fn from(s: &str) -> Self {
        SearchTerm(s.to_string())
    }
}
