import { create } from "zustand";
import type { HardwareUsage } from "@/lib/api";

export type GaugeUnit = "npu" | "gpu" | "cpu";

/** One minute at one sample per second. */
export const HISTORY_LENGTH = 60;

export type UsageSeries = { npu: number[]; gpu: number[]; cpu: number[]; watts: number[] };

type UsageHistory = UsageSeries & { latest: HardwareUsage | null };

const empty = (): UsageHistory => ({ npu: [], gpu: [], cpu: [], watts: [], latest: null });

/** Recent load per unit, for sparklines. Not persisted. */
export const useUsageHistory = create<UsageHistory>()(empty);

const push = (series: number[], value: number | null) =>
  [...series, value ?? 0].slice(-HISTORY_LENGTH);

export function recordUsage(usage: HardwareUsage) {
  useUsageHistory.setState((state) => ({
    npu: push(state.npu, usage.npuPercent),
    gpu: push(state.gpu, usage.gpuPercent),
    cpu: push(state.cpu, usage.cpuPercent),
    // Whole-system power, when the device meters it.
    watts: usage.power ? push(state.watts, usage.power.systemWatts) : state.watts,
    latest: usage,
  }));
}

export function resetUsageHistory() {
  useUsageHistory.setState(empty());
}
