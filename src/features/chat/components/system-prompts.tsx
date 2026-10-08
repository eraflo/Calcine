import { BookmarkPlus, X } from "lucide-react";
import { type FormEvent, useState } from "react";
import { Button } from "@/components/ui/button";
import { Field, Input, Textarea } from "@/components/ui/field";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import { cn } from "@/lib/utils";
import { messages } from "../messages";
import { useChat } from "../store";

/** The system prompt, with prompts saved for reuse as chips below it. */
export function SystemPromptField() {
  const t = useT(messages);
  const tc = useT(common);
  const systemPrompt = useChat((state) => state.settings.systemPrompt);
  const setSettings = useChat((state) => state.setSettings);
  const prompts = useChat((state) => state.prompts);
  const savePrompt = useChat((state) => state.savePrompt);
  const removePrompt = useChat((state) => state.removePrompt);
  const [naming, setNaming] = useState(false);
  const [name, setName] = useState("");

  const text = systemPrompt.trim();
  const save = (event: FormEvent) => {
    event.preventDefault();
    if (!name.trim() || !text) return;
    savePrompt(name.trim(), text);
    setName("");
    setNaming(false);
  };

  return (
    <Field label={t("systemPrompt")} htmlFor="system-prompt">
      <Textarea
        id="system-prompt"
        value={systemPrompt}
        onChange={(event) => setSettings({ systemPrompt: event.target.value })}
        placeholder={t("systemPromptPlaceholder")}
        className="min-h-24"
      />
      {naming ? (
        <form onSubmit={save} className="flex gap-1.5">
          <Input
            autoFocus
            value={name}
            onChange={(event) => setName(event.target.value)}
            onKeyDown={(event) => event.key === "Escape" && setNaming(false)}
            placeholder={t("promptName")}
            aria-label={t("promptName")}
            maxLength={40}
          />
          <Button type="submit" size="sm" className="h-8" disabled={!name.trim()}>
            {t("save")}
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            onClick={() => setNaming(false)}
            aria-label={tc("cancel")}
          >
            <X />
          </Button>
        </form>
      ) : (
        <div className="flex flex-wrap gap-1">
          {prompts.map((prompt) => (
            <span
              key={prompt.id}
              className={cn(
                "inline-flex items-center rounded-sm border text-[11px]",
                prompt.text === text && "border-primary/60 text-primary",
              )}
            >
              <button
                type="button"
                onClick={() => setSettings({ systemPrompt: prompt.text })}
                className="max-w-36 truncate px-1.5 py-0.5 hover:text-foreground"
                title={prompt.text}
              >
                {prompt.name}
              </button>
              <button
                type="button"
                onClick={() => removePrompt(prompt.id)}
                className="rounded-sm p-0.5 text-muted-foreground hover:bg-accent hover:text-foreground"
                aria-label={t("removePrompt", { name: prompt.name })}
              >
                <X className="size-3" />
              </button>
            </span>
          ))}
          <button
            type="button"
            onClick={() => setNaming(true)}
            disabled={!text}
            className="inline-flex items-center gap-1 rounded-sm px-1.5 py-0.5 text-[11px] text-muted-foreground hover:text-foreground disabled:opacity-40 disabled:hover:text-muted-foreground"
            title={text ? undefined : t("savePromptEmpty")}
          >
            <BookmarkPlus className="size-3" />
            {t("savePrompt")}
          </button>
        </div>
      )}
    </Field>
  );
}
