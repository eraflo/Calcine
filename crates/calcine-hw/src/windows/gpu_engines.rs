//! NPU and GPU load from the `GPU Engine` performance counters, the source
//! Task Manager uses.
//!
//! Windows reports both the Adreno GPU and the Hexagon NPU (an MCDM
//! compute-only adapter) as graphics adapters. Each counter instance is one
//! process on one engine of one adapter:
//! `pid_1234_luid_0x00000000_0x0001684E_phys_0_eng_4_engtype_Compute`.
//! An adapter whose engines are all `Compute` is the NPU; one with 3D and
//! video engines is the GPU.

// The PDH C API needs `unsafe`; it is confined to the `Query` type below.
#![allow(unsafe_code)]

use std::collections::{HashMap, HashSet};

use windows_sys::Win32::System::Performance::{
    PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_VALID_DATA, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE,
    PDH_HCOUNTER, PDH_HQUERY, PDH_MORE_DATA, PdhAddEnglishCounterW, PdhCloseQuery,
    PdhCollectQueryData, PdhGetFormattedCounterArrayW, PdhOpenQueryW,
};

/// Load of the NPU and GPU, 0-100.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct EngineLoad {
    pub npu: f32,
    pub gpu: f32,
}

/// A live `GPU Engine(*)\Utilization Percentage` query. Values are rates, so
/// each read measures the time since the previous one.
#[derive(Debug)]
pub(crate) struct GpuEngines {
    query: Query,
}

impl GpuEngines {
    pub(crate) fn open() -> Result<Self, String> {
        let query = Query::open(r"\GPU Engine(*)\Utilization Percentage")?;
        // Prime the query: the first read needs a previous sample.
        let _ = query.read();
        Ok(Self { query })
    }

    pub(crate) fn read(&self) -> Result<EngineLoad, String> {
        Ok(aggregate(self.query.read()?))
    }
}

/// Sum each engine over processes, take each adapter's busiest engine, and
/// sort adapters into NPU and GPU.
fn aggregate(samples: impl IntoIterator<Item = (String, f64)>) -> EngineLoad {
    let mut engines: HashMap<(String, String), f64> = HashMap::new();
    let mut engine_types: HashMap<String, HashSet<String>> = HashMap::new();
    for (name, value) in samples {
        let Some(instance) = Instance::parse(&name) else {
            continue;
        };
        *engines
            .entry((instance.adapter.clone(), instance.engine))
            .or_default() += value;
        engine_types
            .entry(instance.adapter)
            .or_default()
            .insert(instance.engine_type);
    }

    let mut load = EngineLoad::default();
    for (adapter, types) in &engine_types {
        let busiest = engines
            .iter()
            .filter(|((engine_adapter, _), _)| engine_adapter == adapter)
            .map(|(_, value)| *value)
            .fold(0.0_f64, f64::max)
            .clamp(0.0, 100.0);
        #[allow(clippy::cast_possible_truncation)]
        let busiest = busiest as f32;
        let compute_only = types.iter().all(|kind| kind == "compute");
        let graphics = types.contains("3d")
            && types
                .iter()
                .any(|kind| kind.starts_with("video") || kind == "compute");
        if compute_only {
            load.npu = load.npu.max(busiest);
        } else if graphics {
            load.gpu = load.gpu.max(busiest);
        }
    }
    load
}

#[derive(Debug, PartialEq)]
struct Instance {
    adapter: String,
    /// `phys_0_eng_4`, unique within the adapter.
    engine: String,
    /// Lowercase: `3d`, `compute`, `videodecode`…
    engine_type: String,
}

impl Instance {
    fn parse(name: &str) -> Option<Self> {
        let lower = name.to_ascii_lowercase();
        let rest = &lower[lower.find("luid_")?..];
        let phys = rest.find("_phys_")?;
        let kind = rest.find("_engtype_")?;
        Some(Self {
            adapter: rest[..phys].to_owned(),
            engine: rest[phys + 1..kind].to_owned(),
            engine_type: rest[kind + "_engtype_".len()..].to_owned(),
        })
    }
}

/// A PDH query holding one wildcard counter.
#[derive(Debug)]
struct Query {
    query: PDH_HQUERY,
    counter: PDH_HCOUNTER,
}

// SAFETY: PDH handles aren't tied to the thread that created them; Calcine
// only uses a query from one thread at a time (behind a mutex).
unsafe impl Send for Query {}

impl Query {
    fn open(path: &str) -> Result<Self, String> {
        let path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        let mut query: PDH_HQUERY = std::ptr::null_mut();
        // SAFETY: a null data source means live counters; `query` is a valid
        // out pointer.
        check(unsafe { PdhOpenQueryW(std::ptr::null(), 0, &raw mut query) })?;
        let mut counter: PDH_HCOUNTER = std::ptr::null_mut();
        // SAFETY: `query` is open and `path` is NUL-terminated UTF-16 that
        // outlives the call.
        let added = unsafe { PdhAddEnglishCounterW(query, path.as_ptr(), 0, &raw mut counter) };
        if let Err(err) = check(added) {
            // SAFETY: `query` is open and closed only here.
            unsafe { PdhCloseQuery(query) };
            return Err(err);
        }
        Ok(Self { query, counter })
    }

    /// Collect a sample and return each instance's value.
    fn read(&self) -> Result<Vec<(String, f64)>, String> {
        // SAFETY: `self.query` stays open for `self`'s lifetime.
        check(unsafe { PdhCollectQueryData(self.query) })?;

        let mut size = 0_u32;
        let mut count = 0_u32;
        // SAFETY: a null buffer with size 0 asks for the required size.
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &raw mut size,
                &raw mut count,
                std::ptr::null_mut(),
            )
        };
        if status != PDH_MORE_DATA {
            // No instances (nothing uses the GPU or NPU): an empty sample.
            return Ok(Vec::new());
        }

        // The buffer holds the items followed by the strings they point to;
        // u64 storage keeps it aligned for the items.
        let mut buffer = vec![0_u64; (size as usize).div_ceil(8)];
        let items = buffer.as_mut_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>();
        // SAFETY: `buffer` is at least `size` bytes and suitably aligned.
        check(unsafe {
            PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &raw mut size,
                &raw mut count,
                items,
            )
        })?;

        let mut samples = Vec::with_capacity(count as usize);
        for index in 0..count as usize {
            // SAFETY: PDH wrote `count` items at the start of `buffer`.
            let item = unsafe { *items.add(index) };
            let status = item.FmtValue.CStatus;
            if status != PDH_CSTATUS_VALID_DATA && status != PDH_CSTATUS_NEW_DATA {
                continue;
            }
            // SAFETY: `szName` points to a NUL-terminated string inside `buffer`.
            let name = unsafe { wide_to_string(item.szName) };
            // SAFETY: PDH_FMT_DOUBLE fills the `doubleValue` member.
            let value = unsafe { item.FmtValue.Anonymous.doubleValue };
            samples.push((name, value));
        }
        Ok(samples)
    }
}

impl Drop for Query {
    fn drop(&mut self) {
        // SAFETY: the query is open and dropped once.
        unsafe { PdhCloseQuery(self.query) };
    }
}

/// # Safety
///
/// `ptr` must be null or point to a NUL-terminated UTF-16 string.
unsafe fn wide_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0;
    // SAFETY: guaranteed NUL-terminated by the caller.
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }
    // SAFETY: `len` u16s before the NUL are readable.
    String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(ptr, len) })
}

fn check(status: u32) -> Result<(), String> {
    if status == 0 {
        Ok(())
    } else {
        Err(format!("performance counter error 0x{status:08X}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(name: &str, value: f64) -> (String, f64) {
        (name.to_owned(), value)
    }

    #[test]
    fn parses_instance_names() {
        let instance =
            Instance::parse("pid_1234_luid_0x00000000_0x0001684E_phys_0_eng_4_engtype_Compute")
                .unwrap();
        assert_eq!(
            instance,
            Instance {
                adapter: "luid_0x00000000_0x0001684e".into(),
                engine: "phys_0_eng_4".into(),
                engine_type: "compute".into(),
            }
        );
        assert!(Instance::parse("_Total").is_none());
    }

    #[test]
    fn sorts_adapters_and_sums_processes_per_engine() {
        // Engines seen on a Snapdragon X Elite: Adreno (3D, video, compute),
        // Hexagon (compute only) and a 3D-only render adapter to ignore.
        let load = aggregate([
            sample("pid_1_luid_0x0_0xA_phys_0_eng_0_engtype_3D", 10.0),
            sample("pid_2_luid_0x0_0xA_phys_0_eng_0_engtype_3D", 15.0),
            sample("pid_1_luid_0x0_0xA_phys_0_eng_3_engtype_VideoDecode", 5.0),
            sample("pid_3_luid_0x0_0xA_phys_0_eng_4_engtype_Compute", 20.0),
            sample("pid_4_luid_0x0_0xB_phys_0_eng_0_engtype_Compute", 60.0),
            sample("pid_5_luid_0x0_0xB_phys_0_eng_0_engtype_Compute", 30.0),
            sample("pid_6_luid_0x0_0xC_phys_0_eng_0_engtype_3D", 99.0),
        ]);
        assert_eq!(
            load,
            EngineLoad {
                npu: 90.0,
                gpu: 25.0
            }
        );
    }

    #[test]
    fn caps_at_one_hundred() {
        let load = aggregate([
            sample("pid_1_luid_0x0_0xB_phys_0_eng_0_engtype_Compute", 80.0),
            sample("pid_2_luid_0x0_0xB_phys_0_eng_0_engtype_Compute", 70.0),
        ]);
        assert!((load.npu - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reads_live_counters() {
        let engines = GpuEngines::open().expect("GPU Engine counters exist on Windows 10+");
        let load = engines.read().unwrap();
        assert!((0.0..=100.0).contains(&load.npu));
        assert!((0.0..=100.0).contains(&load.gpu));
    }
}
