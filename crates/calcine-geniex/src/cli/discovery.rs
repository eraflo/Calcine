//! Locating the `geniex` executable.

use std::path::{Path, PathBuf};

/// Find GenieX, in order of preference:
///
/// 1. `custom`, when the user picked an executable in Settings;
/// 2. the official installer location, which Calcine installs and updates;
/// 3. `geniex` on `PATH`.
///
/// The official location wins over `PATH` because other tools also ship a
/// `geniex` command (the Python bindings install one) with different
/// subcommands.
pub fn locate(custom: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = custom {
        return path.is_file().then(|| path.to_path_buf());
    }
    official_install_path()
        .filter(|path| path.is_file())
        .or_else(|| which::which("geniex").ok())
}

/// Where the official GenieX installer puts the executable.
///
/// Windows: `%LOCALAPPDATA%\GenieX CLI\geniex.exe` (per-user Inno Setup install).
pub fn official_install_path() -> Option<PathBuf> {
    if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA")
            .map(|dir| PathBuf::from(dir).join("GenieX CLI").join("geniex.exe"))
    } else {
        None
    }
}

/// GenieX's data directory: `custom`, else `GENIEX_DATADIR`, else
/// `~/.cache/geniex` (where `geniex list` reports models are cached).
pub fn data_dir(custom: Option<&Path>) -> Option<PathBuf> {
    if let Some(dir) = custom {
        return Some(dir.to_path_buf());
    }
    if let Some(dir) = std::env::var_os("GENIEX_DATADIR").filter(|dir| !dir.is_empty()) {
        return Some(PathBuf::from(dir));
    }
    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })?;
    Some(PathBuf::from(home).join(".cache").join("geniex"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_custom_binary_is_not_replaced_by_auto_discovery() {
        let missing = Path::new("this/path/does/not/exist/geniex.exe");
        assert_eq!(locate(Some(missing)), None);
    }

    #[test]
    fn custom_data_dir_wins() {
        let custom = Path::new("D:/models");
        assert_eq!(data_dir(Some(custom)), Some(custom.to_path_buf()));
    }

    #[test]
    fn existing_custom_binary_is_used_as_is() {
        let this_file = Path::new(file!()).canonicalize();
        if let Ok(path) = this_file {
            assert_eq!(locate(Some(&path)), Some(path));
        }
    }
}
