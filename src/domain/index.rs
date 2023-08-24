pub struct SearchTerm(String);

impl SearchTerm {
    pub fn inner(&self) -> &str {
        &self.0
    }

    pub fn wildcard() -> Self {
        Self("*".to_string())
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

    pub fn map<T, F: FnOnce(Self) -> T>(self, f: F) -> T {
        f(self)
    }
}

impl From<&str> for SearchTerm {
    fn from(s: &str) -> Self {
        SearchTerm(s.to_string())
    }
}
