/**
 * Chat requests:
 *
 * - `settings`: the settings, their defaults and presets
 * - `build`: the request body for the settings and the conversation
 * - `json`: schemas and JSON replies (structured output)
 * - `stop`: stop sequences
 * - `curl`: the same request as a curl command
 */

export {
  buildChatRequest,
  type ChatTurn,
  type MediaPart,
  supportsComputeChoice,
  supportsMedia,
} from "./build";
export { toCurl } from "./curl";
export { parseSchema, prettyJson } from "./json";
export {
  activePreset,
  type ChatSettings,
  DEFAULT_SETTINGS,
  needsDraftModel,
  type OutputFormat,
  POWER_MODES,
  type PowerMode,
  PRESETS,
  type Preset,
  SPEC_TYPES,
  type SpecType,
  type VisionCompute,
  withDefaults,
} from "./settings";
export { stopIndex } from "./stop";
