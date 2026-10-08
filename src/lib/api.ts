import { isTauri } from "@tauri-apps/api/core";
import type { ApiError, ErrorKind } from "./bindings";

export type * from "./bindings";
export { commands, events } from "./bindings";

/** The local API base URL other apps connect to. */
export const API_BASE_URL = "http://127.0.0.1:18181/v1";

/** A typed backend error, thrown so TanStack Query can surface it. */
export class CalcineError extends Error {
  readonly kind: ErrorKind | "not_in_tauri";

  constructor(kind: ErrorKind | "not_in_tauri", message: string) {
    super(message);
    this.name = "CalcineError";
    this.kind = kind;
  }
}

type CommandResult<T> = { status: "ok"; data: T } | { status: "error"; error: ApiError };

/** Unwrap a tauri-specta result, throwing a {@link CalcineError} on failure. */
export async function unwrap<T>(call: () => Promise<CommandResult<T>>): Promise<T> {
  ensureTauri();
  const result = await call();
  if (result.status === "error") {
    throw new CalcineError(result.error.kind, result.error.message);
  }
  return result.data;
}

/** Call a command that can't fail. */
export async function call<T>(command: () => Promise<T>): Promise<T> {
  ensureTauri();
  return command();
}

function ensureTauri() {
  if (!isTauri()) {
    throw new CalcineError(
      "not_in_tauri",
      "Calcine's backend is only available in the desktop app. Run `bun run app`.",
    );
  }
}
