//! Against the real hubs. Run with `cargo test -p calcine-hub -- --ignored`.

use calcine_core::models::{ModelDirectory, ModelReference, ModelType};
use calcine_hub::{HubConfig, HubDirectory};

fn directory() -> HubDirectory {
    HubDirectory::new(HubConfig::default()).expect("HTTPS client")
}

#[tokio::test]
#[ignore = "needs network"]
async fn searches_hugging_face() {
    let models = directory().search("qwen3", 5).await.unwrap();
    assert_ne!(models.len(), 0);
    assert!(models.iter().all(|model| model.name.contains('/')));
}

#[tokio::test]
#[ignore = "needs network"]
async fn lists_precisions_of_a_vlm() {
    let reference = ModelReference::parse("ggml-org/SmolVLM-256M-Instruct-GGUF").unwrap();
    let details = directory().details(&reference).await.unwrap();
    assert_eq!(details.model_type, ModelType::Vlm);
    assert!(details.precisions.iter().any(|p| p.recommended));
}

#[tokio::test]
#[ignore = "needs network"]
async fn unknown_repo_is_a_clear_error() {
    let reference = ModelReference::parse("calcine-test/does-not-exist-GGUF").unwrap();
    let err = directory().details(&reference).await.unwrap_err();
    assert!(err.to_string().contains("doesn't have"), "{err}");
}

#[tokio::test]
#[ignore = "needs network"]
async fn lists_ai_hub_chipsets() {
    let chipsets = directory().chipsets().await.unwrap();
    assert_ne!(chipsets.len(), 0);
}
