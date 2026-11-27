import type { BatterySaver } from "@/lib/api";

/** As the gateway starts: off, then no thinking and short replies. */
export const DEFAULT_SAVER: Required<BatterySaver> = {
  enabled: false,
  noThinking: true,
  maxReplyTokens: 512,
  powerMode: "power_saver",
};

/** Reply lengths offered, in tokens (`null`: as asked). */
export const REPLY_LIMITS = [256, 512, 1024, 2048, null] as const;

/** The saved settings, with the defaults for what's missing. */
export function saverOf(saved: BatterySaver | undefined): Required<BatterySaver> {
  return { ...DEFAULT_SAVER, ...saved };
}
