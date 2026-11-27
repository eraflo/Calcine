//! What the model reads of the documents for a message.

use std::sync::{Arc, PoisonError};

use calcine_core::Services;
use calcine_docs::{Candidate, DocumentContext, StoredDocument};
use serde_json::json;

use super::Documents;

/// How much each of the user's last messages counts when ranking passages,
/// newest first.
const QUERY_WEIGHTS: [f32; 3] = [1.0, 0.5, 0.25];

impl Documents {
    /// What the model reads of `ids` for the user's last `queries` (newest
    /// first), within `budget` tokens.
    pub async fn context(
        &self,
        services: &Services,
        model: &str,
        ids: &[String],
        queries: &[String],
        budget: u32,
    ) -> calcine_core::Result<DocumentContext> {
        let store = self.store.clone();
        let wanted = ids.to_vec();
        let documents: Vec<StoredDocument> = tokio::task::spawn_blocking(move || {
            wanted
                .iter()
                .filter_map(|id| {
                    store
                        .get(id)
                        .inspect_err(|err| tracing::warn!(%err, %id, "missing document"))
                        .ok()
                })
                .collect()
        })
        .await
        .map_err(|err| calcine_core::Error::Io(std::io::Error::other(err)))?;

        let mut counts = Vec::with_capacity(documents.len());
        for document in &documents {
            counts.push(self.token_counts(services, model, document).await?);
        }
        let candidates: Vec<Candidate<'_>> = documents
            .iter()
            .zip(&counts)
            .map(|(document, tokens)| Candidate {
                document,
                tokens: tokens.as_slice(),
            })
            .collect();
        let queries: Vec<(&str, f32)> = queries
            .iter()
            .zip(QUERY_WEIGHTS)
            .map(|(query, weight)| (query.as_str(), weight))
            .collect();
        Ok(calcine_docs::select(&candidates, &queries, budget))
    }

    /// Tokens of each passage of `document` for `model`, counted once.
    async fn token_counts(
        &self,
        services: &Services,
        model: &str,
        document: &StoredDocument,
    ) -> calcine_core::Result<Arc<Vec<u32>>> {
        let key = (model.to_owned(), document.info.id.clone());
        if let Some(counts) = self
            .counts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&key)
        {
            return Ok(counts.clone());
        }
        let messages: Vec<_> = document
            .passages
            .iter()
            .map(|passage| json!({ "role": "user", "content": passage.text }))
            .collect();
        let count = services
            .context
            .count(model, &json!({ "messages": messages }))
            .await?;
        let counts = Arc::new(count.per_message);
        self.counts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key, counts.clone());
        Ok(counts)
    }
}
