import { Cpu, Image, Type } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Tooltip } from "@/components/ui/tooltip";
import { useT } from "@/i18n";
import type { ModelType, Runtime } from "@/lib/api";
import { messages } from "../messages";

const RUNTIMES = {
  qairt: { label: "qairt", hint: "qairtHint", tone: "npu" },
  llama_cpp: { label: "llamaCpp", hint: "llamaCppHint", tone: "gpu" },
  unknown: { label: "unknownRuntime", hint: "unknownRuntimeHint", tone: "neutral" },
} as const satisfies Record<
  Runtime,
  { label: keyof (typeof messages)["en"]; hint: keyof (typeof messages)["en"]; tone: string }
>;

export function RuntimeBadge({ runtime }: { runtime: Runtime }) {
  const t = useT(messages);
  const { label, tone, hint } = RUNTIMES[runtime];
  return (
    <Tooltip content={t(hint)}>
      <Badge tone={tone} tabIndex={0}>
        <Cpu aria-hidden="true" />
        {t(label)}
      </Badge>
    </Tooltip>
  );
}

export function ModelTypeBadge({ type }: { type: ModelType }) {
  const t = useT(messages);
  if (type === "unknown") return <Badge>{t("typeUnknown")}</Badge>;
  const vision = type === "vlm";
  return (
    <Badge>
      {vision ? <Image aria-hidden="true" /> : <Type aria-hidden="true" />}
      {vision ? t("vision") : t("text")}
    </Badge>
  );
}
