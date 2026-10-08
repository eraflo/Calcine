//! The few strings the shell shows itself (tray menu, notifications), in the
//! language the UI picked.

use std::sync::RwLock;

use serde::Deserialize;
use specta::Type;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    #[default]
    En,
    Fr,
}

/// The UI language, set by the webview at startup and when it changes.
#[derive(Debug, Default)]
pub struct Locale(RwLock<Language>);

impl Locale {
    pub fn get(&self) -> Language {
        *self
            .0
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn set(&self, language: Language) {
        *self
            .0
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = language;
    }
}

/// A notification title and body for a finished job.
pub struct JobMessages {
    pub done: &'static str,
    pub failed: &'static str,
}

pub struct Strings {
    pub open: &'static str,
    pub quit: &'static str,
    pub download: JobMessages,
    pub import: JobMessages,
    pub geniex_update: JobMessages,
    /// After a model name: "… is ready to use."
    pub model_ready: &'static str,
    /// After a version: "… is installed."
    pub version_installed: &'static str,
}

impl Language {
    pub fn strings(self) -> Strings {
        match self {
            Self::En => Strings {
                open: "Open Calcine",
                quit: "Quit Calcine",
                download: JobMessages {
                    done: "Download finished",
                    failed: "Download failed",
                },
                import: JobMessages {
                    done: "Import finished",
                    failed: "Import failed",
                },
                geniex_update: JobMessages {
                    done: "GenieX updated",
                    failed: "GenieX update failed",
                },
                model_ready: "is ready to use.",
                version_installed: "is installed.",
            },
            Self::Fr => Strings {
                open: "Ouvrir Calcine",
                quit: "Quitter Calcine",
                download: JobMessages {
                    done: "Téléchargement terminé",
                    failed: "Échec du téléchargement",
                },
                import: JobMessages {
                    done: "Import terminé",
                    failed: "Échec de l'import",
                },
                geniex_update: JobMessages {
                    done: "GenieX mis à jour",
                    failed: "Échec de la mise à jour de GenieX",
                },
                model_ready: "est prêt à l'emploi.",
                version_installed: "est installé.",
            },
        }
    }
}
