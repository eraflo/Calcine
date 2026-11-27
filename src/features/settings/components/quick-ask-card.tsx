import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Keyboard } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Kbd } from "@/components/ui/kbd";
import { Switch } from "@/components/ui/switch";
import { keysOf, shortcutOf } from "@/features/quick-ask";
import { defineMessages, useT } from "@/i18n";
import { call, commands, unwrap } from "@/lib/api";

const strings = defineMessages({
  en: {
    title: "Quick question",
    description:
      "A shortcut opens a small window over any app to ask a model something, about what you copied if you like. It works while Calcine is in the tray.",
    enable: "Open with a shortcut",
    shortcut: "Shortcut",
    change: "Change",
    press: "Press the new keys…",
    pressHint: "With Ctrl, Alt, Shift or Win. Esc cancels.",
    cancel: "Cancel",
    taken: "{keys} is already used by another app: choose other keys.",
    invalid: "{keys} can't be used as a shortcut: choose other keys.",
  },
  fr: {
    title: "Question rapide",
    description:
      "Un raccourci ouvre une petite fenêtre par-dessus n'importe quelle appli pour poser une question à un modèle, sur le texte copié si vous voulez. Ça marche même quand Calcine est dans la zone de notification.",
    enable: "Ouvrir avec un raccourci",
    shortcut: "Raccourci",
    change: "Modifier",
    press: "Appuyez sur les nouvelles touches…",
    pressHint: "Avec Ctrl, Alt, Maj ou Win. Échap annule.",
    cancel: "Annuler",
    taken: "{keys} est déjà utilisé par une autre appli : choisissez d'autres touches.",
    invalid: "{keys} ne peut pas servir de raccourci : choisissez d'autres touches.",
  },
});

const DEFAULT_SHORTCUT = "Ctrl+Shift+Space";
const quickAskQuery = { queryKey: ["quick-ask"], queryFn: () => call(commands.quickAskStatus) };

/** The shortcut that opens the quick question window. */
export function QuickAskCard() {
  const t = useT(strings);
  const queryClient = useQueryClient();
  const status = useQuery(quickAskQuery);
  const [recording, setRecording] = useState(false);
  const save = useMutation({
    mutationFn: (shortcut: string | null) => unwrap(() => commands.setQuickAskShortcut(shortcut)),
    onSuccess: () => setRecording(false),
    onSettled: () => queryClient.invalidateQueries({ queryKey: quickAskQuery.queryKey }),
  });
  const shortcut = status.data?.shortcut ?? null;
  // The shell says why in English: "<keys> is already used by another app"
  // or "<keys> isn't a shortcut Calcine understands".
  const reason = save.error?.message ?? status.data?.error;
  const keys = reason?.split(" ")[0] ?? "";
  const problem = !reason
    ? null
    : reason.includes("already used")
      ? t("taken", { keys })
      : reason.includes("isn't a shortcut")
        ? t("invalid", { keys })
        : reason;

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("title")}</CardTitle>
        <CardDescription>{t("description")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div className="flex items-center justify-between gap-4">
          <label htmlFor="quick-ask" className="text-sm">
            {t("enable")}
          </label>
          <Switch
            id="quick-ask"
            checked={shortcut !== null}
            disabled={status.isPending || save.isPending}
            onCheckedChange={(on) => save.mutate(on ? DEFAULT_SHORTCUT : null)}
          />
        </div>
        {shortcut !== null && (
          <div className="flex items-center justify-between gap-4">
            <span className="text-sm">{t("shortcut")}</span>
            {recording ? (
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  // biome-ignore lint/a11y/noAutofocus: the keys are typed right after clicking Change.
                  autoFocus
                  onKeyDown={(event) => {
                    event.preventDefault();
                    if (event.key === "Escape") return setRecording(false);
                    const pressed = shortcutOf(event);
                    if (pressed) save.mutate(pressed);
                  }}
                  onBlur={() => setRecording(false)}
                  className="flex h-8 items-center gap-2 rounded-md border border-primary/60 bg-primary/5 px-3 text-xs outline-none"
                >
                  <Keyboard className="size-3.5" />
                  <span className="flex flex-col items-start">
                    <span>{t("press")}</span>
                    <span className="text-[10px] text-muted-foreground">{t("pressHint")}</span>
                  </span>
                </button>
                <Button variant="ghost" size="sm" onMouseDown={() => setRecording(false)}>
                  {t("cancel")}
                </Button>
              </div>
            ) : (
              <div className="flex items-center gap-2">
                <span className="flex items-center gap-1">
                  {keysOf(shortcut).map((key) => (
                    <Kbd key={key}>{key}</Kbd>
                  ))}
                </span>
                <Button variant="secondary" size="sm" onClick={() => setRecording(true)}>
                  {t("change")}
                </Button>
              </div>
            )}
          </div>
        )}
        {problem && <p className="text-xs text-warning">{problem}</p>}
      </CardContent>
    </Card>
  );
}
