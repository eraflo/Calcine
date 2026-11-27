import { useNavigate } from "@tanstack/react-router";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";
import { newId, useChat } from "@/features/chat/store";
import { HANDOFF_EVENT, type Handoff } from "./lib/handoff";

/** In Calcine's window: a quick question continued here becomes a conversation. */
export function useQuickAskHandoff() {
  const navigate = useNavigate();
  useEffect(() => {
    if (!isTauri()) return;
    const listening = getCurrentWindow().listen<Handoff>(HANDOFF_EVENT, ({ payload }) => {
      const chat = useChat.getState();
      const id = chat.create(payload.modelId);
      chat.addMessage(id, { id: newId(), role: "user", content: payload.question });
      chat.addMessage(id, {
        id: newId(),
        role: "assistant",
        content: payload.answer,
        ...(payload.stats ? { stats: payload.stats } : {}),
      });
      void navigate({ to: "/chat" });
    });
    return () => {
      void listening.then((unlisten) => unlisten());
    };
  }, [navigate]);
}
