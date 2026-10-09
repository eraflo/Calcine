//! What differs between Windows and Linux when running GenieX's programs.
//!
//! - Release assets are named by platform (`windows`, `linux`); GenieX only
//!   ships ARM64 builds.
//! - On Windows, programs run without a console window popping up.
//! - On Linux, GenieX's executables look for their shared libraries in the
//!   current directory (their `RUNPATH` is `.` plus build paths), so they
//!   get their folder in `LD_LIBRARY_PATH`; and they die with Calcine, the
//!   way the Windows Job Object kills them.

use std::path::Path;

use tokio::process::Command;

/// The platform name in GenieX's release assets.
pub const NAME: &str = if cfg!(windows) { "windows" } else { "linux" };

/// GenieX only ships ARM64 builds.
pub const ARCH: &str = "arm64";

/// The compute unit `geniex serve` uses when a request doesn't name one.
/// GenieX loads llama.cpp models on the Hexagon NPU by default, which fails
/// on Linux machines that don't expose the compute DSP (no FastRPC device):
/// those default to the CPU. Requests can still ask for another unit.
pub fn default_compute() -> Option<&'static str> {
    if cfg!(target_os = "linux")
        && !Path::new("/dev/fastrpc-cdsp").exists()
        && !Path::new("/dev/fastrpc-cdsp-secure").exists()
    {
        Some("cpu")
    } else {
        None
    }
}

/// Set up `command` as described above; `libraries` is the folder holding
/// the program's shared libraries.
pub fn prepare(command: &mut Command, libraries: &Path) {
    #[cfg(windows)]
    {
        let _ = libraries;
        // CREATE_NO_WINDOW
        command.creation_flags(0x0800_0000);
    }
    #[cfg(unix)]
    {
        command.env(
            "LD_LIBRARY_PATH",
            library_path(libraries, std::env::var_os("LD_LIBRARY_PATH")),
        );
        #[cfg(target_os = "linux")]
        die_with_parent(command);
    }
}

/// `dir` first, then what was already there.
#[cfg(unix)]
fn library_path(dir: &Path, existing: Option<std::ffi::OsString>) -> std::ffi::OsString {
    let mut path = dir.as_os_str().to_owned();
    if let Some(existing) = existing.filter(|existing| !existing.is_empty()) {
        path.push(":");
        path.push(existing);
    }
    path
}

/// Ask the kernel to kill the child when Calcine exits, even after a crash.
///
/// The signal fires when the *thread* that spawned the child ends, so spawn
/// from async tasks (runtime workers live as long as Calcine), never from
/// `spawn_blocking`, whose threads end when idle.
#[cfg(target_os = "linux")]
#[allow(unsafe_code)]
fn die_with_parent(command: &mut Command) {
    let parent = std::process::id();
    // SAFETY: `prctl` and `getppid` are async-signal-safe, as `pre_exec`
    // requires, and touch no memory of the parent.
    unsafe {
        command.pre_exec(move || {
            if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            // Calcine died before the line above: don't outlive it. (No
            // allocation here, so a raw error.)
            if libc::getppid().cast_unsigned() != parent {
                return Err(std::io::Error::from_raw_os_error(libc::ESRCH));
            }
            Ok(())
        });
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn puts_the_program_folder_first() {
        assert_eq!(library_path(Path::new("/opt/geniex"), None), "/opt/geniex");
        assert_eq!(
            library_path(Path::new("/opt/geniex"), Some("/usr/lib/x".into())),
            "/opt/geniex:/usr/lib/x"
        );
        assert_eq!(
            library_path(Path::new("/opt/geniex"), Some("".into())),
            "/opt/geniex"
        );
    }
}
