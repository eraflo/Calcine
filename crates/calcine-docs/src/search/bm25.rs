//! Okapi BM25: how well each passage matches the user's messages.

use std::collections::HashMap;

use super::terms::terms;

/// Term frequency saturation.
const K1: f32 = 1.2;
/// Length normalization.
const B: f32 = 0.75;

/// BM25 statistics of a set of passages.
#[derive(Debug, Clone, Default)]
pub struct Index {
    /// Term counts of each passage.
    counts: Vec<HashMap<String, u32>>,
    lengths: Vec<u32>,
    average_length: f32,
    /// How many passages contain each term.
    passages_with: HashMap<String, u32>,
}

impl Index {
    pub fn new<'a>(texts: impl IntoIterator<Item = &'a str>) -> Self {
        let mut index = Self::default();
        for text in texts {
            let mut counts: HashMap<String, u32> = HashMap::new();
            let words = terms(text);
            for word in &words {
                *counts.entry(word.clone()).or_default() += 1;
            }
            for word in counts.keys() {
                *index.passages_with.entry(word.clone()).or_default() += 1;
            }
            index
                .lengths
                .push(u32::try_from(words.len()).unwrap_or(u32::MAX));
            index.counts.push(counts);
        }
        let total: f64 = index.lengths.iter().map(|&length| f64::from(length)).sum();
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let average = (total / index.lengths.len().max(1) as f64) as f32;
        index.average_length = average;
        index
    }

    pub fn len(&self) -> usize {
        self.counts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }

    /// A score per passage for weighted queries: the current message
    /// weighs most, earlier ones help follow-up questions ("and the
    /// second one?").
    pub fn scores(&self, queries: &[(&str, f32)]) -> Vec<f32> {
        let mut weights: HashMap<String, f32> = HashMap::new();
        for (query, weight) in queries {
            for word in terms(query) {
                let entry = weights.entry(word).or_default();
                *entry = entry.max(*weight);
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let passages = self.counts.len() as f32;
        let average = self.average_length.max(1.0);
        self.counts
            .iter()
            .zip(&self.lengths)
            .map(|(counts, &length)| {
                #[allow(clippy::cast_precision_loss)]
                let length = length as f32;
                weights
                    .iter()
                    .filter_map(|(word, weight)| {
                        #[allow(clippy::cast_precision_loss)]
                        let frequency = *counts.get(word)? as f32;
                        #[allow(clippy::cast_precision_loss)]
                        let with = self.passages_with.get(word).copied().unwrap_or(0) as f32;
                        let idf = (1.0 + (passages - with + 0.5) / (with + 0.5)).ln();
                        let saturation = frequency * (K1 + 1.0)
                            / (frequency + K1 * (1.0 - B + B * length / average));
                        Some(weight * idf * saturation)
                    })
                    .sum()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_matching_passage_ranks_first() {
        let passages = [
            "The company was founded in 1998 in Lyon by two engineers.",
            "The 2027 budget plans 4 million euros for research and hiring.",
            "Our offices are in Lyon, Paris and Grenoble.",
        ];
        let index = Index::new(passages);
        let scores = index.scores(&[("What is the budget for 2027?", 1.0)]);
        let best = scores
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap()
            .0;
        assert_eq!(best, 1, "{scores:?}");
        assert_eq!(scores[0], 0.0);
    }

    #[test]
    fn accents_and_plurals_dont_matter() {
        let index = Index::new(["Les dépenses prévues", "Autre chose"]);
        let scores = index.scores(&[("depense prevue", 1.0)]);
        assert!(scores[0] > 0.0 && scores[1] == 0.0, "{scores:?}");
    }

    #[test]
    fn earlier_messages_count_less() {
        let index = Index::new(["apples are red", "bananas are yellow"]);
        let scores = index.scores(&[("bananas", 1.0), ("apples", 0.3)]);
        assert!(scores[1] > scores[0] && scores[0] > 0.0, "{scores:?}");
    }

    #[test]
    fn a_question_of_stopwords_matches_nothing() {
        let index = Index::new(["anything at all"]);
        assert_eq!(index.scores(&[("what is it about?", 1.0)]), vec![0.0]);
        assert!(Index::new(Vec::<&str>::new()).is_empty());
    }
}
