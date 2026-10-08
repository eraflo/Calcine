import type { DiskSpace, MemoryInfo } from "@/lib/api";

/**
 * Running a model takes roughly its file size plus room for the context
 * (KV cache) and the runtime itself.
 */
const RUNTIME_OVERHEAD = 1.2;

/** Keep this much free on the drive after a download. */
const DISK_MARGIN = 1024 ** 3;

export type MemoryFit = "fits" | "tight" | "too_big";

/**
 * Whether a model of `sizeBytes` fits in memory:
 *
 * - `fits`: in what's free right now
 * - `tight`: only after closing other apps (within 80 % of all memory)
 * - `too_big`: not on this device
 */
export function memoryFit(sizeBytes: number, memory: MemoryInfo): MemoryFit {
  const needed = sizeBytes * RUNTIME_OVERHEAD;
  if (needed <= memory.availableBytes) return "fits";
  if (needed <= memory.totalBytes * 0.8) return "tight";
  return "too_big";
}

/** The largest model that fits in free memory. */
export function comfortableModelBytes(memory: MemoryInfo): number {
  return Math.floor(memory.availableBytes / RUNTIME_OVERHEAD);
}

/** Whether a download of `sizeBytes` leaves some room on the drive. */
export function fitsOnDisk(sizeBytes: number, disk: DiskSpace): boolean {
  return sizeBytes + DISK_MARGIN <= disk.availableBytes;
}
