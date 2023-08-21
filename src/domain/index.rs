pub struct SearchTerm(String);

impl SearchTerm {
    pub fn inner(&self) -> &str {
        &self.0
    }
}

pub struct SearchTermLimit(usize);

impl SearchTermLimit {
    pub fn inner(&self) -> usize {
        self.0
    }
}

impl From<&str> for SearchTerm {
    fn from(s: &str) -> Self {
        SearchTerm(s.to_string())
    }
}
