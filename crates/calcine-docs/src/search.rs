//! Keyword ranking of passages (Okapi BM25), in French and English.
//!
//! Embedding models would match meaning rather than words, but GenieX
//! doesn't run them yet; BM25 needs nothing but the text and finds the
//! passages that share the question's rarer words, which is what most
//! questions about a document need.

use std::collections::HashMap;

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// Term frequency saturation.
const K1: f32 = 1.2;
/// Length normalization.
const B: f32 = 0.75;

/// Words too common to tell passages apart, accents already removed.
const STOPWORDS: &[&str] = &[
    // English
    "a",
    "about",
    "above",
    "after",
    "again",
    "all",
    "also",
    "am",
    "an",
    "and",
    "any",
    "are",
    "as",
    "at",
    "be",
    "because",
    "been",
    "before",
    "being",
    "between",
    "both",
    "but",
    "by",
    "can",
    "could",
    "did",
    "do",
    "does",
    "doing",
    "down",
    "during",
    "each",
    "few",
    "for",
    "from",
    "further",
    "had",
    "has",
    "have",
    "having",
    "he",
    "her",
    "here",
    "hers",
    "him",
    "his",
    "how",
    "i",
    "if",
    "in",
    "into",
    "is",
    "it",
    "its",
    "just",
    "me",
    "more",
    "most",
    "my",
    "no",
    "nor",
    "not",
    "now",
    "of",
    "off",
    "on",
    "once",
    "only",
    "or",
    "other",
    "our",
    "out",
    "over",
    "own",
    "same",
    "she",
    "should",
    "so",
    "some",
    "such",
    "than",
    "that",
    "the",
    "their",
    "them",
    "then",
    "there",
    "these",
    "they",
    "this",
    "those",
    "through",
    "to",
    "too",
    "under",
    "until",
    "up",
    "very",
    "was",
    "we",
    "were",
    "what",
    "when",
    "where",
    "which",
    "while",
    "who",
    "whom",
    "why",
    "will",
    "with",
    "would",
    "you",
    "your",
    "yours",
    "tell",
    "please",
    "summarize",
    "summarise",
    "summary",
    "overview",
    "document",
    "documents",
    "file",
    "text",
    // French
    "au",
    "aux",
    "avec",
    "ce",
    "ces",
    "cet",
    "cette",
    "dans",
    "de",
    "des",
    "du",
    "elle",
    "elles",
    "en",
    "est",
    "et",
    "etre",
    "eu",
    "il",
    "ils",
    "je",
    "la",
    "le",
    "les",
    "leur",
    "leurs",
    "lui",
    "ma",
    "mais",
    "me",
    "meme",
    "mes",
    "moi",
    "mon",
    "ne",
    "nos",
    "notre",
    "nous",
    "on",
    "ou",
    "par",
    "pas",
    "pour",
    "qu",
    "que",
    "qui",
    "quoi",
    "quel",
    "quelle",
    "quels",
    "quelles",
    "sa",
    "se",
    "ses",
    "son",
    "sont",
    "sur",
    "ta",
    "te",
    "tes",
    "toi",
    "ton",
    "tu",
    "un",
    "une",
    "vos",
    "votre",
    "vous",
    "y",
    "a",
    "ai",
    "as",
    "avons",
    "avez",
    "ont",
    "suis",
    "es",
    "sommes",
    "etes",
    "fait",
    "faire",
    "peux",
    "peut",
    "dit",
    "dis",
    "comment",
    "combien",
    "pourquoi",
    "quand",
    "donne",
    "explique",
    "moi",
    "fichier",
    "texte",
    "resume",
    "resumer",
    "synthese",
    "synthetise",
    "apercu",
    "c",
    "d",
    "j",
    "l",
    "m",
    "n",
    "s",
    "t",
];

/// The searchable words of `text`: lowercase, without accents or
/// stopwords, plurals folded (`budgets` → `budget`).
pub fn terms(text: &str) -> Vec<String> {
    let folded: String = text
        .nfkd()
        .filter(|character| !is_combining_mark(*character))
        .flat_map(char::to_lowercase)
        .collect();
    folded
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty() && !STOPWORDS.contains(word))
        .filter(|word| word.chars().count() > 1 || word.chars().all(|c| c.is_ascii_digit()))
        .map(stem)
        .collect()
}

/// Plural folding, the same in French and English for most words.
fn stem(word: &str) -> String {
    let length = word.chars().count();
    if length > 3
        && (word.ends_with('s') || word.ends_with('x'))
        && !word.ends_with("ss")
        && !word.ends_with("us")
        && !word.ends_with("is")
    {
        word[..word.len() - 1].to_owned()
    } else {
        word.to_owned()
    }
}

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
    fn terms_are_folded() {
        assert_eq!(
            terms("Quel est le Budget prévu pour l'Été 2027 ?"),
            vec!["budget", "prevu", "ete", "2027"]
        );
        assert_eq!(
            terms("The budgets of the projects"),
            vec!["budget", "project"]
        );
        // `us`, `is`, `ss` endings aren't plurals.
        assert_eq!(
            terms("status analysis class"),
            vec!["status", "analysis", "class"]
        );
    }

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
