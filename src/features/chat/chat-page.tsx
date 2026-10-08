import { useQuery } from "@tanstack/react-query";
import { PanelRight } from "lucide-react";
import { useState } from "react";
import { ErrorState } from "@/components/calcine/feedback/error-state";
import { Button } from "@/components/ui/button";
import { Tooltip } from "@/components/ui/tooltip";
import { modelsQuery } from "@/features/library/api";
import { Composer } from "./components/composer";
import { ConversationList } from "./components/conversation-list";
import { ModelPicker } from "./components/model-picker";
import { SettingsPanel } from "./components/settings-panel";
import { Thread } from "./components/thread";
import { useChat } from "./store";
import { useSend } from "./use-send";

export function ChatPage() {
  const models = useQuery(modelsQuery);
  const conversations = useChat((state) => state.conversations);
  const activeId = useChat((state) => state.activeId);
  const lastModelId = useChat((state) => state.lastModelId);
  const create = useChat((state) => state.create);
  const setModel = useChat((state) => state.setModel);
  const { send, stop, streaming, ready } = useSend();
  const [showSettings, setShowSettings] = useState(true);

  const installed = models.data ?? [];
  const active = conversations.find((conversation) => conversation.id === activeId);
  const fallbackModel =
    installed.find((model) => model.name === lastModelId)?.name ?? installed[0]?.name;
  const modelId = active?.modelId ?? fallbackModel;
  const model = installed.find((candidate) => candidate.name === modelId);
  const missingModel = Boolean(active && !model && models.isSuccess);

  const sendText = (text: string) => {
    if (!modelId) return;
    const id = active?.id ?? create(modelId);
    void send(id, text);
  };

  if (models.isError) {
    return (
      <div className="p-8">
        <ErrorState error={models.error} onRetry={() => models.refetch()} />
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0">
      <ConversationList onNew={() => modelId && create(modelId)} />
      <section className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-12 shrink-0 items-center gap-3 border-b px-4">
          <ModelPicker
            models={installed}
            value={modelId}
            onChange={(next) =>
              active ? setModel(active.id, next) : useChat.setState({ lastModelId: next })
            }
          />
          {missingModel && (
            <span className="text-xs text-warning">
              This conversation's model was removed. Pick another one.
            </span>
          )}
          <Tooltip content={showSettings ? "Hide settings" : "Show settings"}>
            <Button
              variant="ghost"
              size="icon"
              className="ml-auto"
              onClick={() => setShowSettings((shown) => !shown)}
              aria-label="Toggle settings"
              aria-pressed={showSettings}
            >
              <PanelRight />
            </Button>
          </Tooltip>
        </header>
        <Thread conversation={active} onSuggestion={sendText} />
        <Composer
          disabled={!model || !ready}
          disabledReason={!ready ? "Calcine's API isn't ready yet" : undefined}
          streaming={streaming}
          onSend={sendText}
          onStop={stop}
        />
      </section>
      {showSettings && <SettingsPanel model={model} modelId={modelId} />}
    </div>
  );
}
