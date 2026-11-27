//! The searchable words of a text.

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use super::stopwords::is_stopword;

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
        .filter(|word| !word.is_empty() && !is_stopword(word))
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
}
