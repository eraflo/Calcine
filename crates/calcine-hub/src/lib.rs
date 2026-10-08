//! Model hubs queried over HTTP: Hugging Face search and precisions, and
//! the Qualcomm AI Hub chipset list.
//!
//! GenieX only shows these in interactive prompts, which Calcine never opens.
//!
//! - `hf`: Hugging Face API payloads
//! - `quant`: precision names, as GenieX derives them from file names
//! - `aihub`: AI Hub `platform.json`

mod aihub;
mod hf;
mod quant;

use std::sync::Mutex;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use calcine_core::models::{
    Chipset, ModelDirectory, ModelHub, ModelReference, RemoteModel, RemoteModelDetails,
};
use calcine_core::{Error, Result};
use reqwest::StatusCode;

/// Where to reach the hubs.
#[derive(Debug, Clone)]
pub struct HubConfig {
    /// Hugging Face, or a mirror (`HF_ENDPOINT`, as GenieX reads it).
    pub hf_endpoint: String,
    /// Qualcomm AI Hub public assets.
    pub aihub_endpoint: String,
    pub timeout: Duration,
}

impl Default for HubConfig {
    fn default() -> Self {
        let hf_endpoint = std::env::var("HF_ENDPOINT")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "https://huggingface.co".to_owned());
        Self {
            hf_endpoint: hf_endpoint.trim_end_matches('/').to_owned(),
            aihub_endpoint:
                "https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-models".to_owned(),
            timeout: Duration::from_secs(20),
        }
    }
}

/// The chipset list changes with AI Hub releases, rarely.
const CHIPSETS_TTL: Duration = Duration::from_hours(1);

/// HTTP client for the hubs.
#[derive(Debug)]
pub struct HubDirectory {
    config: HubConfig,
    http: reqwest::Client,
    chipsets: Mutex<Option<(Instant, Vec<Chipset>)>>,
}

impl HubDirectory {
    /// # Errors
    ///
    /// If the TLS backend can't be initialised.
    pub fn new(config: HubConfig) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(concat!("Calcine/", env!("CARGO_PKG_VERSION")))
            .timeout(config.timeout)
            .build()
            .map_err(|err| Error::Network(format!("couldn't set up HTTPS: {err}")))?;
        Ok(Self {
            config,
            http,
            chipsets: Mutex::new(None),
        })
    }

    async fn get(&self, url: &str, query: &[(&str, &str)], what: &str) -> Result<String> {
        let response = self
            .http
            .get(url)
            .query(query)
            .send()
            .await
            .map_err(|err| {
                Error::Network(format!("couldn't reach {what}: {}", root_cause(&err)))
            })?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::Network(match status {
                // Hugging Face answers 401 for missing repos too, to keep
                // private ones secret.
                StatusCode::NOT_FOUND | StatusCode::UNAUTHORIZED => {
                    format!("{what} doesn't have this model, or it's private")
                }
                StatusCode::FORBIDDEN => {
                    format!("{what} needs you to accept this model's license on its website")
                }
                StatusCode::TOO_MANY_REQUESTS => format!("{what} is rate limiting; try again soon"),
                _ => format!("{what} answered {status}"),
            }));
        }
        response
            .text()
            .await
            .map_err(|err| Error::Network(format!("{what} reply was cut short: {err}")))
    }
}

#[async_trait]
impl ModelDirectory for HubDirectory {
    async fn search(&self, query: &str, limit: u32) -> Result<Vec<RemoteModel>> {
        let query = query.trim();
        let limit = limit.clamp(1, 100).to_string();
        let mut params = vec![
            ("filter", "gguf"),
            ("sort", "downloads"),
            ("direction", "-1"),
            ("limit", limit.as_str()),
        ];
        if !query.is_empty() {
            params.push(("search", query));
        }
        params.extend(hf::SEARCH_FIELDS.iter().map(|field| ("expand[]", *field)));
        let url = format!("{}/api/models", self.config.hf_endpoint);
        hf::parse_search(&self.get(&url, &params, "Hugging Face").await?)
    }

    async fn details(&self, reference: &ModelReference) -> Result<RemoteModelDetails> {
        let on_hugging_face = match reference.hub {
            ModelHub::HuggingFace => true,
            // GenieX resolves bare `qualcomm/…` names on AI Hub.
            ModelHub::Auto => !reference.name.to_ascii_lowercase().starts_with("qualcomm/"),
            ModelHub::AiHub | ModelHub::ModelScope | ModelHub::DockerHub | ModelHub::LocalFs => {
                false
            }
        };
        if !on_hugging_face {
            return Err(Error::NotImplemented("Listing precisions for this hub"));
        }
        let url = format!("{}/api/models/{}", self.config.hf_endpoint, reference.name);
        hf::parse_details(&self.get(&url, &[("blobs", "true")], "Hugging Face").await?)
    }

    async fn chipsets(&self) -> Result<Vec<Chipset>> {
        if let Some((at, chipsets)) = self.chipsets.lock().expect("not poisoned").as_ref()
            && at.elapsed() < CHIPSETS_TTL
        {
            return Ok(chipsets.clone());
        }
        let url = format!(
            "{}/releases/latest/platform.json",
            self.config.aihub_endpoint
        );
        let body = self.get(&url, &[], "Qualcomm AI Hub").await?;
        let chipsets = aihub::parse_chipsets(&body, aihub::host_os_types())?;
        *self.chipsets.lock().expect("not poisoned") = Some((Instant::now(), chipsets.clone()));
        Ok(chipsets)
    }
}

/// The innermost error, which says what actually went wrong (DNS, TLS…).
fn root_cause(err: &(dyn std::error::Error + 'static)) -> String {
    let mut current = err;
    while let Some(source) = current.source() {
        current = source;
    }
    current.to_string()
}
