//! Putting `calcine-cli` on the user's `PATH` (Windows), so terminals find
//! it. Done here rather than in the NSIS installer: NSIS strings stop at
//! 1024 characters, and a longer `PATH` would be cut short.
//!
//! Only the user's own `PATH` (`HKCU\Environment`) is touched, keeping its
//! type (`REG_EXPAND_SZ`, so `%VARIABLES%` keep working).

/// `path` with `dir` appended, or `None` when it's already there.
pub fn with_dir(path: &str, dir: &str) -> Option<String> {
    if contains(path, dir) {
        return None;
    }
    let path = path.trim_end_matches(';');
    Some(if path.is_empty() {
        dir.to_owned()
    } else {
        format!("{path};{dir}")
    })
}

/// `path` without `dir`, or `None` when it isn't there.
pub fn without_dir(path: &str, dir: &str) -> Option<String> {
    if !contains(path, dir) {
        return None;
    }
    Some(
        path.split(';')
            .filter(|entry| !entry.is_empty() && !same(entry, dir))
            .collect::<Vec<_>>()
            .join(";"),
    )
}

pub fn contains(path: &str, dir: &str) -> bool {
    path.split(';').any(|entry| same(entry, dir))
}

fn same(entry: &str, dir: &str) -> bool {
    let normalize = |text: &str| text.trim().trim_end_matches(['\\', '/']).to_lowercase();
    !entry.trim().is_empty() && normalize(entry) == normalize(dir)
}

#[cfg(windows)]
pub use self::registry::{add, is_added, remove};

#[cfg(windows)]
mod registry {
    // The registry and window messages need `unsafe`; it stays here.
    #![allow(unsafe_code)]

    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{
        HKEY_CURRENT_USER, REG_EXPAND_SZ, REG_VALUE_TYPE, RRF_NOEXPAND, RRF_RT_REG_EXPAND_SZ,
        RRF_RT_REG_SZ, RegGetValueW, RegSetKeyValueW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
    };

    const ENVIRONMENT: &str = "Environment";

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(Some(0)).collect()
    }

    /// The value and its type; `None` when it doesn't exist yet.
    fn read(name: &str) -> Result<Option<(String, REG_VALUE_TYPE)>, String> {
        let (key, name) = (wide(ENVIRONMENT), wide(name));
        let flags = RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ | RRF_NOEXPAND;
        let mut kind: REG_VALUE_TYPE = 0;
        let mut size = 0_u32;
        // SAFETY: valid NUL-terminated strings; a null buffer asks for the size.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                flags,
                &raw mut kind,
                std::ptr::null_mut(),
                &raw mut size,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if status != ERROR_SUCCESS {
            return Err(format!("can't read the PATH (error {status})"));
        }
        let mut buffer = vec![0_u16; (size as usize).div_ceil(2)];
        // SAFETY: `buffer` holds `size` bytes.
        let status = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                flags,
                &raw mut kind,
                buffer.as_mut_ptr().cast(),
                &raw mut size,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(format!("can't read the PATH (error {status})"));
        }
        let end = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        Ok(Some((String::from_utf16_lossy(&buffer[..end]), kind)))
    }

    fn write(name: &str, value: &str, kind: REG_VALUE_TYPE) -> Result<(), String> {
        let (key, name, data) = (wide(ENVIRONMENT), wide(name), wide(value));
        let bytes = u32::try_from(data.len() * 2).map_err(|_| "the PATH is too long")?;
        // SAFETY: valid NUL-terminated strings; `data` is `bytes` long.
        let status = unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                name.as_ptr(),
                kind,
                data.as_ptr().cast(),
                bytes,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(format!("can't change the PATH (error {status})"));
        }
        // Tell Explorer, so new terminals see it without signing out.
        let environment = wide(ENVIRONMENT);
        // SAFETY: broadcasting a string pointer that outlives the call.
        unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                environment.as_ptr() as isize,
                SMTO_ABORTIFHUNG,
                5000,
                std::ptr::null_mut(),
            );
        }
        Ok(())
    }

    /// Add `dir` to the user variable `name` (`Path`). `false` when it was
    /// already there.
    pub fn add(name: &str, dir: &str) -> Result<bool, String> {
        let (current, kind) = read(name)?.unwrap_or_default();
        let kind = if kind == 0 { REG_EXPAND_SZ } else { kind };
        match super::with_dir(&current, dir) {
            Some(updated) => write(name, &updated, kind).map(|()| true),
            None => Ok(false),
        }
    }

    /// Remove `dir` from the user variable `name`. `false` when it wasn't there.
    pub fn remove(name: &str, dir: &str) -> Result<bool, String> {
        let Some((current, kind)) = read(name)? else {
            return Ok(false);
        };
        match super::without_dir(&current, dir) {
            Some(updated) => write(name, &updated, kind).map(|()| true),
            None => Ok(false),
        }
    }

    pub fn is_added(name: &str, dir: &str) -> Result<bool, String> {
        Ok(read(name)?.is_some_and(|(current, _)| super::contains(&current, dir)))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// Uses its own variable: the real `Path` is never touched.
        #[test]
        fn adds_and_removes_a_folder() {
            let name = format!("CalcineTestPath{}", std::process::id());
            write(&name, r"C:\Tools;%USERPROFILE%\bin", REG_EXPAND_SZ).unwrap();
            assert!(add(&name, r"C:\Apps\Calcine").unwrap());
            assert!(!add(&name, r"c:\apps\calcine\").unwrap(), "already there");
            assert!(is_added(&name, r"C:\Apps\Calcine").unwrap());
            let (value, kind) = read(&name).unwrap().unwrap();
            assert_eq!(value, r"C:\Tools;%USERPROFILE%\bin;C:\Apps\Calcine");
            assert_eq!(kind, REG_EXPAND_SZ);
            assert!(remove(&name, r"C:\Apps\Calcine").unwrap());
            assert_eq!(
                read(&name).unwrap().unwrap().0,
                r"C:\Tools;%USERPROFILE%\bin"
            );
            assert!(!remove(&name, r"C:\Apps\Calcine").unwrap());
            // Clean up: an empty value is harmless, but leave nothing behind.
            delete(&name);
        }

        fn delete(name: &str) {
            use windows_sys::Win32::System::Registry::RegDeleteKeyValueW;
            let (key, name) = (wide(ENVIRONMENT), wide(name));
            // SAFETY: valid NUL-terminated strings.
            unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), name.as_ptr()) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_once_and_removes_every_copy() {
        assert_eq!(with_dir("", r"C:\A").as_deref(), Some(r"C:\A"));
        assert_eq!(with_dir(r"C:\X;", r"C:\A").as_deref(), Some(r"C:\X;C:\A"));
        assert_eq!(with_dir(r"C:\X;c:\a\", r"C:\A"), None);
        assert_eq!(
            without_dir(r"C:\X;C:\A;;C:\Y;c:\a", r"C:\A").as_deref(),
            Some(r"C:\X;C:\Y")
        );
        assert_eq!(without_dir(r"C:\X", r"C:\A"), None);
        assert!(!contains(";;", ""));
    }
}
