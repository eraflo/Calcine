//! Smoke tests against the GenieX installed on this machine.
//!
//! Ignored by default (CI runners have no GenieX). Run them on a Snapdragon
//! device with `cargo test -p calcine-geniex -- --ignored`.

use calcine_core::traits::{ModelCatalog, ModelStore, RuntimeManager};
use calcine_geniex::Geniex;

#[tokio::test]
#[ignore = "needs a local GenieX install"]
async fn reads_runtime_info() {
    let info = Geniex::default()
        .info()
        .await
        .expect("GenieX should be installed");
    assert!(
        info.cli_version.starts_with('v'),
        "unexpected version {}",
        info.cli_version
    );
}

#[tokio::test]
#[ignore = "needs a local GenieX install"]
async fn lists_cached_models() {
    let models = Geniex::default()
        .list()
        .await
        .expect("`geniex list` should succeed");
    for model in models {
        assert_ne!(model.name, "");
    }
}

#[tokio::test]
#[ignore = "needs a local GenieX install"]
async fn reads_the_chipset() {
    let chipset = Geniex::default()
        .chipset()
        .await
        .expect("`geniex config get` should succeed");
    assert_ne!(chipset.as_deref(), Some(""));
}

#[tokio::test]
#[ignore = "needs a local GenieX install and network access"]
async fn lists_the_ai_hub_catalog_for_this_device() {
    let catalog = Geniex::default()
        .aihub(false)
        .await
        .expect("`geniex model list` should succeed");
    assert!(catalog.chipset.is_some());
    assert_ne!(catalog.models, []);
}
