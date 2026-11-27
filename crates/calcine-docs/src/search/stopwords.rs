//! Words too common to search for, accents already removed.

/// Words too common to tell passages apart, accents already removed.
pub(super) fn is_stopword(word: &str) -> bool {
    ENGLISH.contains(&word) || FRENCH.contains(&word) || REQUESTS.contains(&word)
}

#[rustfmt::skip]
const ENGLISH: &[&str] = &[
    "a", "about", "above", "after", "again", "all", "also", "am", "an", "and", "any",
    "are", "as", "at", "be", "because", "been", "before", "being", "between", "both",
    "but", "by", "can", "could", "did", "do", "does", "doing", "down", "during", "each",
    "few", "for", "from", "further", "had", "has", "have", "having", "he", "her",
    "here", "hers", "him", "his", "how", "i", "if", "in", "into", "is", "it", "its",
    "just", "me", "more", "most", "my", "no", "nor", "not", "now", "of", "off", "on",
    "once", "only", "or", "other", "our", "out", "over", "own", "same", "she", "should",
    "so", "some", "such", "than", "that", "the", "their", "them", "then", "there",
    "these", "they", "this", "those", "through", "to", "too", "under", "until", "up",
    "very", "was", "we", "were", "what", "when", "where", "which", "while", "who",
    "whom", "why", "will", "with", "would", "you", "your", "yours",
];

#[rustfmt::skip]
const FRENCH: &[&str] = &[
    "au", "aux", "avec", "ce", "ces", "cet", "cette", "dans", "de", "des", "du", "elle",
    "elles", "en", "est", "et", "etre", "eu", "il", "ils", "je", "la", "le", "les",
    "leur", "leurs", "lui", "ma", "mais", "me", "meme", "mes", "moi", "mon", "ne",
    "nos", "notre", "nous", "on", "ou", "par", "pas", "pour", "qu", "que", "qui",
    "quoi", "quel", "quelle", "quels", "quelles", "sa", "se", "ses", "son", "sont",
    "sur", "ta", "te", "tes", "toi", "ton", "tu", "un", "une", "vos", "votre", "vous",
    "y", "a", "ai", "as", "avons", "avez", "ont", "suis", "es", "sommes", "etes",
    "fait", "faire", "peux", "peut", "c", "d", "j", "l", "m", "n", "s", "t",
];

/// How people ask about a document ("summarize this file"): not what it's about.
#[rustfmt::skip]
const REQUESTS: &[&str] = &[
    "tell", "please", "summarize", "summarise", "summary", "overview", "document",
    "documents", "file", "text", "dit", "dis", "comment", "combien", "pourquoi",
    "quand", "donne", "explique", "fichier", "texte", "resume", "resumer", "synthese",
    "synthetise", "apercu",
];
