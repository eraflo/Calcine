import { useQuery } from "@tanstack/react-query";
import { emitTo } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ArrowUpRight, LoaderCircle, Square } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { CopyButton } from "@/components/calcine/actions/copy-button";
import { LogoMark } from "@/components/calcine/brand/logo";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/field";
import { MarkdownBody } from "@/features/chat/components/message/markdown-body";
import { Stats } from "@/features/chat/components/message/stats";
import { modelsQuery } from "@/features/library/api";
import { useLanguage, useT } from "@/i18n";
import { commands } from "@/lib/api";
import { useTheme } from "@/stores/theme";
import { HANDOFF_EVENT, type Handoff } from "../lib/handoff";
import { chatPreferences, type QuickAction, quickPrompt } from "../lib/prompt";
import { messages } from "../messages";
import { useQuickAskStore } from "../store";
import { useQuickAnswer } from "../use-quick-answer";
import { CopiedBar } from "./copied-bar";

/** Sent by the shell each time the window shows. */
const SHOWN_EVENT = "quick-ask:shown";

/** The quick question window: a question, the copied text, an answer. */
export function QuickAskWindow() {
  const t = useT(messages);
  const { data: models = [], refetch } = useQuery(modelsQuery);
  const chosen = useQuickAskStore((state) => state.modelId);
  const setModelId = useQuickAskStore((state) => state.setModelId);
  const { answer, ask, stop, ready } = useQuickAnswer();
  const [question, setQuestion] = useState("");
  const [copied, setCopied] = useState<string | null>(null);
  const [includeCopied, setIncludeCopied] = useState(false);
  const input = useRef<HTMLTextAreaElement>(null);

  const preferred = chosen ?? chatPreferences().modelId;
  const modelId = models.some((m) => m.name === preferred) ? preferred : models[0]?.name;
  const model = models.find((candidate) => candidate.name === modelId);
  const busy = answer?.busy ?? false;

  // Each time it shows: what's copied now, the newest settings, the input ready.
  const refresh = useCallback(() => {
    void useTheme.persist.rehydrate();
    void useLanguage.persist.rehydrate();
    void refetch();
    void commands.copiedText().then((text) => {
      setCopied(text);
      setIncludeCopied(false);
    });
    input.current?.focus();
    input.current?.select();
  }, [refetch]);
  useEffect(() => {
    refresh();
    const listening = getCurrentWindow().listen(SHOWN_EVENT, refresh);
    return () => {
      void listening.then((unlisten) => unlisten());
    };
  }, [refresh]);

  const send = (text: string, withCopied: boolean) => {
    if (!modelId || !text.trim() || busy) return;
    void ask(model, modelId, quickPrompt(text, withCopied ? copied : null));
  };
  const runAction = (action: QuickAction) => {
    setIncludeCopied(true);
    setQuestion(t(`prompt_${action}`));
    send(t(`prompt_${action}`), true);
  };
  const handOff = async () => {
    if (!answer) return;
    const handoff: Handoff = {
      modelId: answer.modelId,
      question: answer.prompt,
      answer: answer.content,
      ...(answer.stats ? { stats: answer.stats } : {}),
    };
    await emitTo("main", HANDOFF_EVENT, handoff);
    await commands.hideQuickAsk(true);
  };

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      if (busy) stop();
      else void commands.hideQuickAsk(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [busy, stop]);

  const problem = models.length === 0 ? t("noModel") : ready ? null : t("notReady");
  return (
    <div className="flex h-screen flex-col overflow-hidden border bg-background text-foreground">
      <header className="flex items-start gap-2.5 border-b px-3 py-2.5" data-tauri-drag-region>
        <LogoMark className="mt-1 size-5 shrink-0" />
        <textarea
          ref={input}
          value={question}
          onChange={(event) => setQuestion(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && !event.shiftKey && !event.nativeEvent.isComposing) {
              event.preventDefault();
              send(question, includeCopied);
            }
          }}
          placeholder={t("placeholder")}
          rows={2}
          className="min-w-0 flex-1 resize-none bg-transparent py-1 text-[15px] outline-none placeholder:text-muted-foreground"
        />
        {models.length > 0 && (
          <Select
            value={modelId ?? ""}
            onChange={(event) => setModelId(event.target.value)}
            aria-label={t("model")}
            className="h-7 w-44 shrink-0 font-mono text-[11px]"
          >
            {models.map((candidate) => (
              <option key={candidate.name} value={candidate.name}>
                {candidate.name}
              </option>
            ))}
          </Select>
        )}
      </header>

      {copied && (
        <CopiedBar
          copied={copied}
          included={includeCopied}
          onIncludedChange={setIncludeCopied}
          onAction={runAction}
          disabled={busy || !modelId || !ready}
        />
      )}

      <main className="min-h-0 flex-1 overflow-y-auto px-4 py-3">
        {problem ? (
          <p className="text-sm text-warning">{problem}</p>
        ) : !answer ? (
          <p className="text-sm text-muted-foreground">{t("empty")}</p>
        ) : (
          <>
            {answer.content ? (
              <MarkdownBody text={answer.content} />
            ) : (
              answer.busy && <LoaderCircle className="size-4 animate-spin text-muted-foreground" />
            )}
            {answer.error && <p className="mt-2 text-sm text-destructive">{answer.error}</p>}
          </>
        )}
      </main>

      <footer className="flex h-10 shrink-0 items-center gap-2 border-t px-3 text-[11px] text-muted-foreground">
        <span>{busy ? t("hintBusy") : t("hint")}</span>
        {answer?.stats && !busy && (
          <span className="truncate">
            · <Stats stats={answer.stats} />
          </span>
        )}
        <div className="ml-auto flex items-center gap-1">
          {busy && (
            <Button variant="ghost" size="sm" className="h-7" onClick={stop}>
              <Square />
              {t("stop")}
            </Button>
          )}
          {answer?.content && !busy && (
            <>
              <CopyButton value={answer.content} className="size-7" />
              <Button variant="secondary" size="sm" className="h-7" onClick={() => void handOff()}>
                <ArrowUpRight />
                {t("continue")}
              </Button>
            </>
          )}
        </div>
      </footer>
    </div>
  );
}
