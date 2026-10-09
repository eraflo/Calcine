//! Sample data, shaped like a Snapdragon X Elite laptop running GenieX v0.8.0.

use std::path::Path;

use calcine_core::hardware::{
    Accelerator, DiskSpace, HardwareInfo, HardwareUsage, MemoryInfo, PowerDraw, Processor,
};
use calcine_core::models::{
    Chipset, ComputeUnit, HubCatalog, HubModel, LocalModel, ModelHub, ModelType, RemoteModel,
    RemoteModelDetails, RemotePrecision, Runtime,
};
use calcine_core::runtime::RuntimeInfo;

pub(crate) const CHIPSET: &str = "Snapdragon X Elite CRD";
const GIB: u64 = 1 << 30;

pub(crate) fn sample_models() -> Vec<LocalModel> {
    vec![
        LocalModel {
            name: "qualcomm/Qwen3-4B".into(),
            size_bytes: 3_182_356_037,
            runtime: Runtime::Qairt,
            model_type: ModelType::Llm,
            precisions: vec!["W4A16".into()],
        },
        LocalModel {
            name: "unsloth/Qwen3-0.6B-GGUF".into(),
            size_bytes: 1_288_490_188,
            runtime: Runtime::LlamaCpp,
            model_type: ModelType::Llm,
            precisions: vec!["Q4_0".into(), "Q8_0".into()],
        },
        LocalModel {
            name: "google/gemma-4-E2B-it-qat-q4_0-gguf".into(),
            size_bytes: 4_294_967_296,
            runtime: Runtime::LlamaCpp,
            model_type: ModelType::Vlm,
            precisions: vec!["Q4_0".into()],
        },
    ]
}

/// The AI Hub catalog for this device, or for every chipset.
pub(crate) fn catalog(all_chipsets: bool) -> HubCatalog {
    let model = |name: &str, model_type, chipsets: &[&str]| HubModel {
        name: name.into(),
        model_type,
        chipsets: if all_chipsets {
            chipsets.iter().map(|chip| (*chip).to_owned()).collect()
        } else {
            Vec::new()
        },
    };
    let mut models = vec![
        model("qualcomm/Qwen3-0.6B", ModelType::Llm, &["x-elite", "8gen3"]),
        model("qualcomm/Qwen3-4B", ModelType::Llm, &["x-elite", "8gen3"]),
        model(
            "qualcomm/Llama-v3.2-1B-Instruct",
            ModelType::Llm,
            &["x-elite"],
        ),
        model(
            "qualcomm/Qwen3-VL-4B-Instruct",
            ModelType::Vlm,
            &["x-elite"],
        ),
    ];
    if all_chipsets {
        models.push(model(
            "qualcomm/Phi-3.5-Mini-Instruct",
            ModelType::Llm,
            &["8gen3"],
        ));
    }
    HubCatalog {
        chipset: (!all_chipsets).then(|| CHIPSET.to_owned()),
        models,
    }
}

pub(crate) fn runtime_info() -> RuntimeInfo {
    RuntimeInfo {
        cli_version: "v0.8.0".into(),
        qairt_version: Some("2.45".into()),
        llama_cpp_hash: Some("9425611".into()),
        binary_path: "mock://geniex".into(),
    }
}

pub(crate) fn hardware(models_dir: Option<&Path>) -> HardwareInfo {
    HardwareInfo {
        cpu: Some(Processor {
            name: "Snapdragon(R) X Elite - X1E78100 - Qualcomm(R) Oryon(TM) CPU".into(),
            cores: 12,
        }),
        memory: MemoryInfo {
            total_bytes: 32 * GIB,
            available_bytes: 18 * GIB,
        },
        accelerators: vec![
            Accelerator {
                unit: ComputeUnit::Npu,
                name: "Snapdragon(R) X Elite - X1E78100 - Qualcomm(R) Hexagon(TM) NPU".into(),
                driver_version: Some("30.0.219.1000".into()),
            },
            Accelerator {
                unit: ComputeUnit::Gpu,
                name: "Qualcomm(R) Adreno(TM) X1-85 GPU".into(),
                driver_version: Some("31.0.133.1".into()),
            },
        ],
        models_disk: models_dir.map(|dir| DiskSpace {
            path: dir.display().to_string(),
            total_bytes: 1024 * GIB,
            available_bytes: 412 * GIB,
        }),
    }
}

/// What the mock device draws doing nothing.
pub(crate) const IDLE_WATTS: f64 = 8.0;

/// Gentle waves so the gauges move: the NPU works in bursts, like during
/// generation.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn usage(seconds: f32) -> HardwareUsage {
    let wave = |period: f32, low: f32, high: f32| {
        let phase = (seconds / period * std::f32::consts::TAU).sin() * 0.5 + 0.5;
        low + (high - low) * phase
    };
    let npu = wave(20.0, -40.0, 85.0).max(0.0);
    // Up to 2 GiB more in use while the NPU works.
    let busy_mib = u64::from((npu / 100.0 * 2048.0) as u16);
    HardwareUsage {
        cpu_percent: wave(7.0, 6.0, 22.0),
        gpu_percent: Some(wave(11.0, 1.0, 9.0)),
        npu_percent: Some(npu),
        memory: MemoryInfo {
            total_bytes: 32 * GIB,
            available_bytes: (18 * GIB).saturating_sub(busy_mib << 20),
        },
        power: Some(PowerDraw {
            system_watts: IDLE_WATTS as f32 + npu / 100.0 * 7.0,
            cpu_watts: Some(wave(7.0, 1.2, 3.5)),
            gpu_watts: Some(wave(11.0, 0.0, 0.4)),
        }),
    }
}

/// Hugging Face search results.
pub(crate) fn hub_models() -> Vec<RemoteModel> {
    let model = |name: &str, model_type, downloads, likes| RemoteModel {
        name: name.into(),
        hub: ModelHub::HuggingFace,
        model_type,
        downloads,
        likes,
        updated_at: Some("2026-09-12T08:00:00.000Z".into()),
        gated: false,
    };
    vec![
        model("unsloth/Qwen3-0.6B-GGUF", ModelType::Llm, 1_204_331, 412),
        model("unsloth/Qwen3-4B-GGUF", ModelType::Llm, 982_114, 388),
        model(
            "bartowski/Llama-3.2-3B-Instruct-GGUF",
            ModelType::Llm,
            640_902,
            251,
        ),
        model(
            "ggml-org/SmolVLM-256M-Instruct-GGUF",
            ModelType::Unknown,
            14_869,
            20,
        ),
        model(
            "ggml-org/SmolVLM-500M-Instruct-GGUF",
            ModelType::Unknown,
            35_220,
            58,
        ),
        model("ggml-org/gemma-3-4b-it-GGUF", ModelType::Vlm, 410_552, 140),
    ]
}

/// Precisions for any Hugging Face name, sized from the parameter count in it.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn hub_details(name: &str) -> RemoteModelDetails {
    let billions = name
        .split(['-', '/'])
        .find_map(|part| {
            let number = part.strip_suffix(['B', 'b'])?;
            number.parse::<f64>().ok()
        })
        .unwrap_or(0.5);
    let vlm = name.to_ascii_lowercase().contains("vl") || name.contains("gemma-3");
    let projector = if vlm { 190_000_000.0 } else { 0.0 };
    let size = |bits: f64| (billions * 1e9 * bits / 8.0 + projector) as u64;
    let precision = |name: &str, bits, recommended| RemotePrecision {
        name: name.into(),
        size_bytes: size(bits),
        recommended,
    };
    RemoteModelDetails {
        name: name.into(),
        hub: ModelHub::HuggingFace,
        model_type: if vlm { ModelType::Vlm } else { ModelType::Llm },
        precisions: vec![
            precision("Q4_0", 4.5, true),
            precision("Q4_K_M", 4.8, false),
            precision("Q8_0", 8.5, false),
            precision("F16", 16.0, false),
        ],
        gated: false,
    }
}

pub(crate) fn chipsets() -> Vec<Chipset> {
    let chipset = |id: &str, device: &str, marketing: &str, alias: &str| Chipset {
        id: id.into(),
        device: device.into(),
        marketing_name: Some(marketing.into()),
        aliases: vec![alias.into()],
    };
    vec![
        chipset(
            "qualcomm-snapdragon-x-elite",
            "Snapdragon X Elite CRD",
            "Snapdragon X Elite",
            "sc8380xp",
        ),
        chipset(
            "qualcomm-snapdragon-x-plus-8-core",
            "Snapdragon X Plus 8-Core CRD",
            "Snapdragon X Plus (8 core)",
            "sc8340xp",
        ),
        chipset(
            "qualcomm-snapdragon-x2-elite",
            "Snapdragon X2 Elite CRD",
            "Snapdragon X2 Elite",
            "sc8480xp",
        ),
    ]
}
