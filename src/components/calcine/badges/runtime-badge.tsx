import { Cpu, Image, Type } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Tooltip } from "@/components/ui/tooltip";
import type { ModelType, Runtime } from "@/lib/api";

const RUNTIMES: Record<Runtime, { label: string; tone: "npu" | "gpu" | "neutral"; hint: string }> =
  {
    qairt: {
      label: "QAIRT · NPU",
      tone: "npu",
      hint: "Pre-compiled Qualcomm AI Hub bundle. Runs natively on the Hexagon NPU.",
    },
    llama_cpp: {
      label: "llama.cpp",
      tone: "gpu",
      hint: "GGUF model. Runs on the NPU, GPU or CPU.",
    },
    unknown: { label: "Unknown runtime", tone: "neutral", hint: "Runtime not recognized." },
  };

export function RuntimeBadge({ runtime }: { runtime: Runtime }) {
  const { label, tone, hint } = RUNTIMES[runtime];
  return (
    <Tooltip content={hint}>
      <Badge tone={tone} tabIndex={0}>
        <Cpu aria-hidden="true" />
        {label}
      </Badge>
    </Tooltip>
  );
}

export function ModelTypeBadge({ type }: { type: ModelType }) {
  if (type === "unknown") return <Badge>Type unknown</Badge>;
  const vision = type === "vlm";
  return (
    <Badge>
      {vision ? <Image aria-hidden="true" /> : <Type aria-hidden="true" />}
      {vision ? "Vision" : "Text"}
    </Badge>
  );
}
