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

pub struct Strings {
    pub open: &'static str,
    pub quit: &'static str,
    pub download: &'static str,
    pub import: &'static str,
    pub finished: &'static str,
    pub failed: &'static str,
    pub ready: &'static str,
}

impl Language {
    pub fn strings(self) -> Strings {
        match self {
            Self::En => Strings {
                open: "Open Calcine",
                quit: "Quit Calcine",
                download: "Download",
                import: "Import",
                finished: "finished",
                failed: "failed",
                ready: "is ready to use.",
            },
            Self::Fr => Strings {
                open: "Ouvrir Calcine",
                quit: "Quitter Calcine",
                download: "Téléchargement",
                import: "Import",
                finished: "terminé",
                failed: "échoué",
                ready: "est prêt à l'emploi.",
            },
        }
    }
}
