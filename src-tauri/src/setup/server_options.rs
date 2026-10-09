//! `geniex serve` options chosen on the Server page, kept across launches.

use std::path::{Path, PathBuf};

use calcine_core::runtime::ServerOptions;

/// Where the options are saved (`server.json` in the app's data folder).
#[derive(Debug, Clone)]
pub struct ServerOptionsFile(PathBuf);

impl ServerOptionsFile {
    pub fn new(data_dir: &Path) -> Self {
        Self(data_dir.join("server.json"))
    }

    /// The saved options, or GenieX's defaults when missing or invalid.
    pub fn load(&self) -> ServerOptions {
        std::fs::read(&self.0)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<ServerOptions>(&bytes).ok())
            .filter(|options| options.validate().is_ok())
            .unwrap_or_default()
    }

    pub fn save(&self, options: ServerOptions) -> std::io::Result<()> {
        if let Some(dir) = self.0.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_vec_pretty(&options).map_err(std::io::Error::other)?;
        std::fs::write(&self.0, json)
    }
}

#[cfg(test)]
mod tests {
    use calcine_core::runtime::ServerOptions;

    use super::ServerOptionsFile;

    #[test]
    fn round_trips_and_falls_back_to_defaults() {
        let dir =
            std::env::temp_dir().join(format!("calcine-server-options-{}", std::process::id()));
        let file = ServerOptionsFile::new(&dir);
        assert_eq!(file.load(), ServerOptions::default());

        let options = ServerOptions {
            keepalive_secs: 900,
            context_size: 8192,
        };
        file.save(options).unwrap();
        assert_eq!(file.load(), options);

        std::fs::write(dir.join("server.json"), br#"{"keepaliveSecs": 1}"#).unwrap();
        assert_eq!(file.load(), ServerOptions::default());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
