//! Smoke tests against the GenieX installed on this machine.
//!
//! Ignored by default (CI runners have no GenieX). Run them on a Snapdragon
//! device with `cargo test -p calcine-geniex -- --ignored`.

use calcine_core::models::{ModelCatalog, ModelStore};
use calcine_core::runtime::RuntimeManager;
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

#[tokio::test]
#[ignore = "needs a local GenieX install"]
async fn serve_starts_answers_and_stops() {
    use calcine_core::runtime::{InferenceServer, ServerOptions, ServerState};
    use calcine_geniex::GeniexServer;

    let server = GeniexServer::new(Geniex::default(), ServerOptions::default());
    let url = server
        .ensure_running()
        .await
        .expect("geniex serve should start");
    assert!(matches!(
        *server.state().borrow(),
        ServerState::Ready { .. }
    ));
    // A second caller reuses the running server.
    assert_eq!(server.ensure_running().await.unwrap(), url);

    let models = reqwest::get(format!("{url}/v1/models"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(models.contains("\"object\":\"list\""), "{models}");

    server.stop().await.unwrap();
    assert_eq!(*server.state().borrow(), ServerState::Stopped);
    assert!(
        reqwest::get(format!("{url}/v1/")).await.is_err(),
        "the server should be gone"
    );
}
