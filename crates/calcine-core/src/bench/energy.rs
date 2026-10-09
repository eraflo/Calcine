//! Energy profiles: a model's speed and energy use on each compute unit, in
//! each power mode.
//!
//! Measured on a Snapdragon X laptop on mains power with GenieX 0.8, the
//! power modes changed neither speed nor energy noticeably, whether set per
//! request or for the server: the compute unit is what makes the
//! difference.
//!
//! Measured through `geniex serve` rather than `geniex-bench`, so each
//! window is exact: the model is loaded in the power mode first, the system
//! is measured idle with it loaded, then the energy meter is read right
//! before and after each generation. What generating costs is the energy
//! above that idle draw, per generated token (the prompt included).

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use specta::Type;

use super::{BenchMeasure, BenchStat};
use crate::hardware::{EnergyReading, HardwareProbe};
use crate::jobs::JobCtx;
use crate::models::{ComputeUnit, Runtime};
use crate::runtime::InferenceServer;
use crate::{Error, Result};

/// GenieX's power modes, fastest first.
pub const POWER_MODES: [&str; 8] = [
    "burst",
    "sustained_high_performance",
    "high_performance",
    "balanced",
    "low_balanced",
    "high_power_saver",
    "power_saver",
    "low_power_saver",
];

/// After loading, before measuring idle: lets the clocks and whatever the
/// load stirred up settle.
const SETTLE: Duration = Duration::from_secs(3);
/// How long idle is measured.
const IDLE: Duration = Duration::from_secs(4);

/// Long enough to keep generating; each repetition starts differently so
/// GenieX reads the whole prompt again instead of reusing its cache.
const PROMPT: &str = "Write a long, detailed story about a lighthouse keeper who finds a \
                      message in a bottle. Describe the coast, the weather and the people of the \
                      village. Keep writing until you are told to stop.";

/// Which model to profile, on which units, in which power modes. Every unit
/// is measured in every mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EnergyRequest {
    pub model: String,
    pub runtime: Runtime,
    /// The NPU for AI Hub models; llama.cpp models can use any unit.
    pub units: Vec<ComputeUnit>,
    pub power_modes: Vec<String>,
    pub generated_tokens: u32,
    pub repetitions: u32,
}

impl EnergyRequest {
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.model.trim().is_empty() {
            return Err("choose a model".into());
        }
        if self.units.is_empty() {
            return Err("choose at least one compute unit".into());
        }
        if self.power_modes.is_empty() {
            return Err("choose at least one power mode".into());
        }
        if let Some(mode) = self
            .power_modes
            .iter()
            .find(|mode| !POWER_MODES.contains(&mode.as_str()))
        {
            return Err(format!("unknown power mode {mode}"));
        }
        if self.runtime == Runtime::Qairt && self.units.iter().any(|unit| *unit != ComputeUnit::Npu)
        {
            return Err("AI Hub models run on the NPU".into());
        }
        if !(64..=2048).contains(&self.generated_tokens) {
            return Err("generate between 64 and 2048 tokens".into());
        }
        if !(1..=10).contains(&self.repetitions) {
            return Err("run between 1 and 10 repetitions".into());
        }
        Ok(())
    }
}

/// What energy a power mode used.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EnergyMeasure {
    /// The whole system, idle with the model loaded, just before.
    pub idle_watts: f64,
    /// The whole system while generating (median).
    pub active_watts: f64,
    /// Energy above idle per generated token, the prompt included (median):
    /// what generating costs.
    pub joules_per_token: f64,
    /// Energy of the whole system per generated token (median): what the
    /// battery sees, the screen and everything else included.
    #[serde(default)]
    pub system_joules_per_token: f64,
}

/// One generation's numbers.
#[derive(Debug, Clone, Copy)]
struct Run {
    watts: f64,
    joules_per_token: f64,
    system_joules_per_token: f64,
    tokens: f64,
    prompt_tokens: f64,
    prompt_ms: f64,
    prefill_tps: f64,
    decode_tps: f64,
}

/// Measure one power mode. Errors when the device has no energy metering.
pub(crate) async fn measure_mode(
    server: &dyn InferenceServer,
    hardware: &dyn HardwareProbe,
    request: &EnergyRequest,
    unit: ComputeUnit,
    mode: &str,
    ctx: &JobCtx,
) -> Result<BenchMeasure> {
    let completion = |max_tokens: u32, run: u32| {
        let mut body = json!({
            "model": request.model,
            "messages": [{ "role": "user", "content": format!("Story {run}. {PROMPT}") }],
            "max_tokens": max_tokens,
            "temperature": 0.7,
            "enable_think": false,
            "power_mode": mode,
        });
        if request.runtime == Runtime::LlamaCpp {
            body["compute"] = json!(unit);
        }
        body
    };
    // Load the model in this power mode, then measure it idle.
    server.complete(&completion(1, 0)).await?;
    pause(SETTLE, ctx).await?;
    let start = energy(hardware).await?;
    pause(IDLE, ctx).await?;
    let idle_watts = start
        .watts_until(&energy(hardware).await?)
        .ok_or_else(|| Error::InvalidInput("the energy meter didn't move".into()))?;

    let mut runs = Vec::new();
    for run in 1..=request.repetitions {
        if ctx.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let before = energy(hardware).await?;
        let reply = server
            .complete(&completion(request.generated_tokens, run))
            .await?;
        let after = energy(hardware).await?;
        runs.push(read_run(&reply, &before, &after, idle_watts)?);
    }
    Ok(measure(&runs, idle_watts))
}

async fn energy(hardware: &dyn HardwareProbe) -> Result<EnergyReading> {
    hardware.energy().await?.ok_or(Error::NotImplemented(
        "energy measurements need a device with energy metering",
    ))
}

async fn pause(duration: Duration, ctx: &JobCtx) -> Result<()> {
    tokio::select! {
        () = tokio::time::sleep(duration) => Ok(()),
        () = ctx.cancelled() => Err(Error::Cancelled),
    }
}

fn read_run(
    reply: &Value,
    before: &EnergyReading,
    after: &EnergyReading,
    idle_watts: f64,
) -> Result<Run> {
    let number = |pointer: &str| reply.pointer(pointer).and_then(Value::as_f64);
    let tokens = number("/usage/completion_tokens").unwrap_or(0.0);
    let watts = before
        .watts_until(after)
        .ok_or_else(|| Error::InvalidInput("the energy meter didn't move".into()))?;
    if tokens < 1.0 {
        return Err(Error::InvalidInput("the model generated nothing".into()));
    }
    let seconds = after.seconds - before.seconds;
    Ok(Run {
        watts,
        joules_per_token: (watts - idle_watts).max(0.0) * seconds / tokens,
        system_joules_per_token: watts * seconds / tokens,
        tokens,
        prompt_tokens: number("/usage/prompt_tokens").unwrap_or(0.0),
        prompt_ms: number("/timings/prompt_ms").unwrap_or(0.0),
        prefill_tps: number("/timings/prompt_per_second").unwrap_or(0.0),
        decode_tps: number("/timings/predicted_per_second").unwrap_or(0.0),
    })
}

fn measure(runs: &[Run], idle_watts: f64) -> BenchMeasure {
    let stat = |pick: fn(&Run) -> f64| BenchStat::of(&runs.iter().map(pick).collect::<Vec<_>>());
    BenchMeasure {
        ttft_ms: stat(|run| run.prompt_ms),
        prefill_tps: stat(|run| run.prefill_tps),
        decode_tps: stat(|run| run.decode_tps),
        generated_tokens: stat(|run| run.tokens).median,
        prompt_tokens: stat(|run| run.prompt_tokens).median,
        geniex_version: String::new(),
        energy: Some(EnergyMeasure {
            idle_watts,
            active_watts: stat(|run| run.watts).median,
            joules_per_token: stat(|run| run.joules_per_token).median,
            system_joules_per_token: stat(|run| run.system_joules_per_token).median,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> EnergyRequest {
        EnergyRequest {
            model: "qualcomm/Qwen3-4B".into(),
            runtime: Runtime::Qairt,
            units: vec![ComputeUnit::Npu],
            power_modes: vec!["burst".into(), "power_saver".into()],
            generated_tokens: 256,
            repetitions: 2,
        }
    }

    #[test]
    fn validates_requests() {
        assert!(request().validate().is_ok());
        let unknown = EnergyRequest {
            power_modes: vec!["turbo".into()],
            ..request()
        };
        assert_eq!(unknown.validate(), Err("unknown power mode turbo".into()));
        let gpu = EnergyRequest {
            units: vec![ComputeUnit::Npu, ComputeUnit::Gpu],
            ..request()
        };
        assert!(gpu.validate().is_err());
        let none = EnergyRequest {
            power_modes: vec![],
            ..request()
        };
        assert!(none.validate().is_err());
    }

    #[test]
    fn energy_per_token_is_above_idle() {
        let reply = json!({
            "usage": { "completion_tokens": 200, "prompt_tokens": 60 },
            "timings": { "prompt_ms": 80.0, "prompt_per_second": 750.0, "predicted_per_second": 50.0 },
        });
        let before = EnergyReading {
            joules: 100.0,
            seconds: 10.0,
        };
        // 4 s at 22 W, 15 W idle: 28 J above idle for 200 tokens.
        let after = EnergyReading {
            joules: 188.0,
            seconds: 14.0,
        };
        let run = read_run(&reply, &before, &after, 15.0).unwrap();
        assert!((run.watts - 22.0).abs() < 1e-9);
        assert!((run.joules_per_token - 0.14).abs() < 1e-9);
        assert!((run.system_joules_per_token - 0.44).abs() < 1e-9);
        let measure = measure(&[run, run], 15.0);
        assert_eq!(measure.decode_tps.median, 50.0);
        assert_eq!(measure.energy.unwrap().active_watts, 22.0);

        let empty = json!({ "usage": { "completion_tokens": 0 } });
        assert!(read_run(&empty, &before, &after, 15.0).is_err());
    }
}
