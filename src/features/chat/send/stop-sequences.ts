/** Stop sequences, applied by the chat for GenieX versions that ignore them. */

import { stopIndex } from "../lib/request";
import { useLiveReply } from "../store";

/**
 * A stream callback that appends to the live reply and, at the first stop
 * sequence, cuts the reply there and aborts the request.
 */
export function stopAtSequences(stops: readonly string[], abort: AbortController) {
  let reached = false;
  const onDelta = (delta: { content?: string; reasoning?: string }) => {
    useLiveReply.getState().append(delta);
    if (!delta.content || stops.length === 0) return;
    const { content } = useLiveReply.getState();
    const index = stopIndex(content, stops);
    if (index === null) return;
    reached = true;
    useLiveReply.setState({ content: content.slice(0, index) });
    abort.abort();
  };
  return { onDelta, reachedStop: () => reached };
}
