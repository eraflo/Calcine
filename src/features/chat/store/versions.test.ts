import { beforeEach, describe, expect, it } from "vitest";
import { useChat } from "./chat";

const conversation = () => {
  const state = useChat.getState();
  return state.conversations.find((c) => c.id === state.activeId);
};

describe("reply versions and edits", () => {
  let id: string;
  beforeEach(() => {
    useChat.setState({ conversations: [], activeId: null });
    id = useChat.getState().create("qualcomm/Qwen3-4B");
    const chat = useChat.getState();
    chat.addMessage(id, { id: "q1", role: "user", content: "First?" });
    chat.addMessage(id, {
      id: "a1",
      role: "assistant",
      content: "One",
      stats: { completionTokens: 3, durationMs: 100 },
    });
    chat.addMessage(id, { id: "q2", role: "user", content: "Second?" });
    chat.addMessage(id, { id: "a2", role: "assistant", content: "Two", error: "boom" });
  });

  it("keeps earlier versions and fills the new one as it's written", () => {
    const chat = useChat.getState();
    chat.newVersion(id, "a2");
    let reply = conversation()?.messages.at(-1);
    expect(reply).toMatchObject({ content: "", version: 1 });
    expect(reply?.error).toBeUndefined();
    expect(reply?.versions).toEqual([{ content: "Two", error: "boom" }, { content: "" }]);

    chat.updateMessage(id, "a2", { content: "Two, again" });
    reply = conversation()?.messages.at(-1);
    expect(reply?.versions?.[1]).toEqual({ content: "Two, again" });

    chat.selectVersion(id, "a2", 0);
    reply = conversation()?.messages.at(-1);
    expect(reply).toMatchObject({ content: "Two", error: "boom", version: 0 });
    chat.selectVersion(id, "a2", 1);
    expect(conversation()?.messages.at(-1)).toMatchObject({ content: "Two, again", version: 1 });
    expect(conversation()?.messages.at(-1)?.error).toBeUndefined();
  });

  it("drops what came after an edited message", () => {
    useChat.getState().setContext(id, { contextFrom: "q2", summary: "earlier" });
    useChat.getState().editMessage(id, "q1", "First, rephrased?");
    const after = conversation();
    expect(after?.messages.map((m) => m.id)).toEqual(["q1"]);
    expect(after?.messages[0]?.content).toBe("First, rephrased?");
    // The model's view started at a dropped message: it starts over.
    expect(after?.contextFrom).toBeUndefined();
    expect(after?.summary).toBeUndefined();
  });
});
