//! Portable probes: CPU, memory and disk (`sysinfo`).

use std::path::Path;

use calcine_core::hardware::{DiskSpace, MemoryInfo, Processor};
use sysinfo::{CpuRefreshKind, Disks, System};

pub(crate) struct SystemInfo {
    pub cpu: Option<Processor>,
    pub memory: MemoryInfo,
}

/// CPU and memory. `fallback_cpu_name` is used when sysinfo can't read the
/// brand string, which is empty on Windows ARM64.
pub(crate) fn read(fallback_cpu_name: Option<&str>) -> SystemInfo {
    let mut system = System::new();
    system.refresh_memory();
    system.refresh_cpu_list(CpuRefreshKind::nothing());

    let cpu = system.cpus().first().map(|cpu| {
        let brand = cpu.brand().trim();
        let name = if brand.is_empty() {
            fallback_cpu_name
        } else {
            Some(brand)
        };
        Processor {
            name: name.unwrap_or("Unknown processor").to_owned(),
            cores: u32::try_from(System::physical_core_count().unwrap_or(system.cpus().len()))
                .unwrap_or(u32::MAX),
        }
    });

    SystemInfo {
        cpu,
        memory: MemoryInfo {
            total_bytes: system.total_memory(),
            available_bytes: system.available_memory(),
        },
    }
}

/// Free space on the drive that holds `dir` (the longest matching mount point).
pub(crate) fn disk_space(dir: &Path) -> Option<DiskSpace> {
    let target = normalize(dir);
    let disks = Disks::new_with_refreshed_list();
    disks
        .list()
        .iter()
        .filter(|disk| target.starts_with(&normalize(disk.mount_point())))
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .map(|disk| DiskSpace {
            path: dir.display().to_string(),
            total_bytes: disk.total_space(),
            available_bytes: disk.available_space(),
        })
}

/// Compare paths case-insensitively on Windows (`c:\` vs `C:\`).
fn normalize(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    if cfg!(windows) {
        text.to_lowercase()
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_compare_case_insensitively_on_windows() {
        if cfg!(windows) {
            assert_eq!(
                normalize(Path::new(r"C:\Users")),
                normalize(Path::new("c:/users"))
            );
        }
    }

    #[test]
    fn falls_back_to_the_given_cpu_name() {
        let info = read(Some("Fallback CPU"));
        let cpu = info.cpu.expect("a CPU");
        assert_ne!(cpu.name, "");
    }
}
