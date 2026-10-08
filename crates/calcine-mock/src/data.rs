//! Sample data, shaped like a Snapdragon X Elite laptop running GenieX v0.8.0.

use std::path::Path;

use calcine_core::hardware::{Accelerator, DiskSpace, HardwareInfo, MemoryInfo, Processor};
use calcine_core::models::{ComputeUnit, HubCatalog, HubModel, LocalModel, ModelType, Runtime};
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
