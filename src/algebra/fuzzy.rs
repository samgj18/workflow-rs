use std::borrow::Cow;
use std::cmp::Ordering;

#[derive(Debug, Clone, Default)]
pub struct Fuzzy<'a> {
    from: Cow<'a, str>,
}

impl<'a> From<&'a str> for Fuzzy<'a> {
    fn from(s: &'a str) -> Self {
        Self { from: s.into() }
    }
}

impl<'a> Fuzzy<'a> {
    fn trigrams(s: &str) -> Vec<(char, char, char)> {
        let padded = format!("  {} ", s);
        let chars = padded.chars().collect::<Vec<_>>();
        let trigrams = chars
            .windows(3)
            .map(|window| (window[0], window[1], window[2]))
            .collect();
        trigrams
    }

    /// Compute a similarity score between two strings based on
    /// the number of common trigrams they share and the length of the strings.
    pub fn compare(&self, to: &str) -> f32 {
        let trigrams_a = Self::trigrams(to);
        let trigrams_b = Self::trigrams(&self.from);

        let common_trigrams = trigrams_a
            .iter()
            .filter(|&t_a| trigrams_b.contains(t_a))
            .count() as f32;

        let string_len = to.chars().count() as f32;
        let similarity = common_trigrams / (string_len + 1.0);

        similarity.clamp(0.0, 1.0)
    }

    /// Search a list of strings and return sorted results based on
    /// the similarity score.
    pub fn search<T: AsRef<str>>(&self, list: &'a [T]) -> Vec<&'a str> {
        let mut res: Vec<(&'a str, f32)> = list
            .iter()
            .map(|value| (value.as_ref(), self.compare(value.as_ref())))
            .collect();

        res.sort_by(|(_, d1), (_, d2)| d2.partial_cmp(d1).unwrap_or(Ordering::Equal));
        res.into_iter().map(|(value, _)| value).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_search() {
        let fuzzy = Fuzzy::from("test");
        let list = vec!["test", "test2", "test3", "test4", "test5"];
        let res = fuzzy.search(&list);
        assert_eq!(res, vec!["test", "test2", "test3", "test4", "test5"]);
    }

    #[test]
    fn test_fuzzy_search_complicated_words() {
        let fuzzy = Fuzzy::from("deg");
        let list = vec!["d", "de", "def", "defg", "defgh"];
        let res = fuzzy.search(&list);
        assert_eq!(res, vec!["de", "d", "def", "defg", "defgh"]);
    }
}
