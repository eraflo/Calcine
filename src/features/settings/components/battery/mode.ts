import { POWER_MODES, type PowerMode } from "@/features/chat/lib/request";

/** The battery saver's mode, when it's on. */
export const isPowerMode = (mode: string | null | undefined): mode is PowerMode =>
  (POWER_MODES as readonly string[]).includes(mode ?? "");
