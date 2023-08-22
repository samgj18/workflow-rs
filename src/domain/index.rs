pub struct SearchTerm(String);

impl SearchTerm {
    pub fn inner(&self) -> &str {
        &self.0
    }
}

pub struct SearchTermLimit(usize);

impl From<usize> for SearchTermLimit {
    fn from(limit: usize) -> Self {
        Self(limit)
    }
}

impl Default for SearchTermLimit {
    fn default() -> Self {
        Self(10)
    }
}

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
