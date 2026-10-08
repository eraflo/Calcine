import type { Translate } from "@/i18n";
import type { Job } from "@/lib/api";
import { formatBytes } from "@/lib/format";
import type { messages } from "../messages";

type TasksT = Translate<(typeof messages)["en"]>;

export const isRunning = (job: Job) => job.state.state === "running";

/** The model a job is about (`owner/model[:precision]`). */
export function jobTitle(job: Job): string {
  return job.kind.model;
}

/** 0–100, or `null` while the total is unknown. */
export function percent(job: Job): number | null {
  const progress = job.progress;
  if (!progress?.totalBytes) return null;
  return Math.min(100, Math.floor((progress.doneBytes / progress.totalBytes) * 100));
}

/** Seconds left at the current speed, or `null` if it can't be estimated. */
export function secondsLeft(job: Job): number | null {
  const progress = job.progress;
  if (!progress?.totalBytes || !progress.bytesPerSecond) return null;
  return Math.max(0, (progress.totalBytes - progress.doneBytes) / progress.bytesPerSecond);
}

/** `45s`, `3m 05s`, `1h 12m`. */
export function formatDuration(seconds: number): string {
  const total = Math.round(seconds);
  if (total < 60) return `${total}s`;
  const minutes = Math.floor(total / 60);
  if (minutes < 60) return `${minutes}m ${String(total % 60).padStart(2, "0")}s`;
  return `${Math.floor(minutes / 60)}h ${String(minutes % 60).padStart(2, "0")}m`;
}

/** `245 MiB of 725 MiB · 17 MiB/s · 30s left` */
export function progressLabel(job: Job, t: TasksT): string {
  const progress = job.progress;
  if (!progress) return t("starting");
  const parts = [
    progress.totalBytes
      ? t("progressOf", {
          done: formatBytes(progress.doneBytes),
          total: formatBytes(progress.totalBytes),
        })
      : formatBytes(progress.doneBytes),
  ];
  if (progress.bytesPerSecond) {
    parts.push(t("perSecond", { speed: formatBytes(progress.bytesPerSecond) }));
  }
  const left = secondsLeft(job);
  if (left !== null && left > 0) parts.push(t("left", { duration: formatDuration(left) }));
  return parts.join(" · ");
}

/** Insert or replace a job, keeping newest first. */
export function upsertJob(jobs: readonly Job[], job: Job): Job[] {
  const index = jobs.findIndex((existing) => existing.id === job.id);
  if (index === -1) return [job, ...jobs].sort((a, b) => b.id - a.id);
  const next = [...jobs];
  next[index] = job;
  return next;
}
