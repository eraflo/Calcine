//! Context windows and token counts, estimated: AI Hub models have 4096
//! tokens, llama.cpp ones use the server option.

use async_trait::async_trait;
use calcine_core::Result;
use calcine_core::context::{ContextMeter, TokenCount};
use calcine_core::models::Runtime;
use serde_json::Value;

use crate::MockBackend;

#[async_trait]
impl ContextMeter for MockBackend {
    async fn compiled_window(&self, model: &str) -> Result<Option<u32>> {
        let name = model.split(':').next().unwrap_or(model);
        Ok(self
            .models()
            .iter()
            .any(|cached| cached.name == name && cached.runtime == Runtime::Qairt)
            .then_some(4096))
    }

    async fn count(&self, _model: &str, request: &Value) -> Result<TokenCount> {
        let per_message = request
            .get("messages")
            .and_then(Value::as_array)
            .map(|messages| {
                messages
                    .iter()
                    .map(|message| {
                        let text = message
                            .get("content")
                            .map(Value::to_string)
                            .unwrap_or_default();
                        u32::try_from(text.len().div_ceil(3)).unwrap_or(u32::MAX) + 5
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(TokenCount {
            per_message,
            overhead: 20,
            exact: false,
        })
    }
}
