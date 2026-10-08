//! Smoke tests against the GenieX installed on this machine.
//!
//! Ignored by default (CI runners have no GenieX). Run them on a Snapdragon
//! device with `cargo test -p calcine-geniex -- --ignored`.

use calcine_core::traits::{ModelStore, RuntimeManager};
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
