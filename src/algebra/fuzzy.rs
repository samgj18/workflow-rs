use std::cmp::Ordering;

use crate::prelude::Search;

pub trait Fuzzy<'a> {
    fn compare(&self, to: &str, from: &str) -> f32;
    fn search<T: AsRef<str>>(&self, from: &str, list: &'a [T]) -> Vec<&'a str>;
}

impl<'a> Fuzzy<'a> for Search {
    /// Compute a similarity score between two strings based on
    /// the number of common trigrams they share and the length of the strings.
    fn compare(&self, to: &str, from: &str) -> f32 {
        let trigrams_a = trigrams(to);
        let trigrams_b = trigrams(self.query().unwrap_or(from));

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
    ///
    /// Calculate the average similarity score between the query and
    /// each string in the list and returns half of the list sorted
    /// by the similarity score.
    fn search<T: AsRef<str>>(&self, from: &str, list: &'a [T]) -> Vec<&'a str> {
        let mut avg = 0.0;
        let mut res: Vec<(&'a str, f32)> = list
            .iter()
            .map(|value| {
                let score = self.compare(value.as_ref(), from);
                avg += score;
                (value.as_ref(), score)
            })
            .collect();

        res.sort_by(|(_, d1), (_, d2)| d2.partial_cmp(d1).unwrap_or(Ordering::Equal));

        let avg = avg / (res.len() as f32);

        res.into_iter()
            .filter(|(_, d)| *d >= avg)
            .map(|(value, _)| value)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_search() {
        let fuzzy = Search::from("test");
        let list = vec!["test", "test2", "test3", "test4", "test5"];
        let res = fuzzy.search("", &list);
        assert_eq!(res, vec!["test"]);
    }

    #[test]
    fn test_fuzzy_search_complicated_words() {
        let fuzzy = Search::from("deg");
        let list = vec!["d", "de", "def", "defg", "defgh"];
        let res = fuzzy.search("", &list);
        assert_eq!(res, vec!["de", "d", "def"]);
    }
}

fn trigrams(s: &str) -> Vec<(char, char, char)> {
    let padded = format!("  {} ", s);
    let chars = padded.chars().collect::<Vec<_>>();
    let trigrams = chars
        .windows(3)
        .map(|window| (window[0], window[1], window[2]))
        .collect();
    trigrams
}
