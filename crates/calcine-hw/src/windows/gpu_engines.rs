//! NPU and GPU load from the `GPU Engine` performance counters, the source
//! Task Manager uses.
//!
//! Windows reports both the Adreno GPU and the Hexagon NPU (an MCDM
//! compute-only adapter) as graphics adapters. Each counter instance is one
//! process on one engine of one adapter:
//! `pid_1234_luid_0x00000000_0x0001684E_phys_0_eng_4_engtype_Compute`.
//! An adapter whose engines are all `Compute` is the NPU; one with 3D and
//! video engines is the GPU.

use std::collections::{HashMap, HashSet};

use super::pdh::Query;

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
