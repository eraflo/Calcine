import type { ModelReference } from "@/lib/api";

/**
 * Whether Calcine can list a model's precisions: Hugging Face repositories,
 * including bare names GenieX resolves there (not `qualcomm/…`, which come
 * from AI Hub pre-quantized).
 */
export function canListPrecisions(reference: Pick<ModelReference, "hub" | "name">): boolean {
  if (reference.hub === "hugging_face") return true;
  return reference.hub === "auto" && !reference.name.toLowerCase().startsWith("qualcomm/");
}
