//! Keyword ranking of passages (Okapi BM25), in French and English.
//!
//! Embedding models would match meaning rather than words, but GenieX
//! doesn't run them yet; BM25 needs nothing but the text and finds the
//! passages that share the question's rarer words, which is what most
//! questions about a document need.
//!
//! - `terms`: the searchable words of a text (accents, plurals, stopwords)
//! - `bm25`: passages scored against weighted queries

mod bm25;
mod stopwords;
mod terms;

pub use bm25::Index;
pub use terms::terms;
