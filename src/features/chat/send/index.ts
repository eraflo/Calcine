/**
 * Sending a message:
 *
 * - `use-send`: the hook, which runs the steps below and streams the reply
 * - `turns`: the user's message, and the history as the API expects it
 * - `documents`: what the model reads of the attached documents
 * - `fit-context`: forgetting and summarizing what no longer fits
 * - `stop-sequences`: cutting the reply at a stop sequence
 */

export { type PendingAttachment, toTurns } from "./turns";
export { useSend } from "./use-send";
