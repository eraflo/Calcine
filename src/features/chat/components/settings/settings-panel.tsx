import type { LocalModel } from "@/lib/api";
import { supportsComputeChoice } from "../../lib/request";
import { useChat } from "../../store";
import { ComputeFields } from "./compute-fields";
import { ContextFields } from "./context-fields";
import { GenerationFields } from "./generation-fields";
import { ModelLoadingSection } from "./model-loading-section";
import { OutputFormatField } from "./output-format-field";
import { RequestPreview } from "./request-preview";
import { SamplingSection } from "./sampling-section";
import { SpeculativeSection } from "./speculative-section";
import { SystemPromptField } from "./system-prompts";

/**
 * Generation options, adapted to what the selected model supports. Each
 * group lives in its own file in this folder.
 */
export function SettingsPanel({
  model,
  modelId,
}: {
  model: LocalModel | undefined;
  modelId: string | undefined;
}) {
  const settings = useChat((state) => state.settings);
  const llamaCpp = supportsComputeChoice(model);
  return (
    <aside className="flex w-72 shrink-0 flex-col gap-5 overflow-y-auto overscroll-contain border-l p-4">
      <SystemPromptField />
      <GenerationFields />
      <ContextFields />
      <OutputFormatField />
      <SamplingSection llamaCpp={llamaCpp} />
      <ComputeFields llamaCpp={llamaCpp} />
      <ModelLoadingSection model={model} llamaCpp={llamaCpp} />
      <SpeculativeSection modelId={modelId} llamaCpp={llamaCpp} />
      {modelId && <RequestPreview model={model} modelId={modelId} settings={settings} />}
    </aside>
  );
}
