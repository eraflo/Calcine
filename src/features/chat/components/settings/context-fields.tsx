import { Switch } from "@/components/ui/switch";
import { useT } from "@/i18n";
import { useChat } from "../../store";
import { settingsMessages as messages } from "./messages";

/** What happens when the conversation outgrows the model's window. */
export function ContextFields() {
  const t = useT(messages);
  const settings = useChat((state) => state.settings);
  const setSettings = useChat((state) => state.setSettings);
  return (
    <>
      <div className="flex items-center justify-between gap-3">
        <label htmlFor="forget-oldest" className="flex flex-col">
          <span className="text-xs font-medium text-muted-foreground">{t("forgetOldest")}</span>
          <span className="text-[11px] text-muted-foreground">{t("forgetOldestHint")}</span>
        </label>
        <Switch
          id="forget-oldest"
          checked={settings.forgetOldest}
          onCheckedChange={(forgetOldest) => setSettings({ forgetOldest })}
        />
      </div>
      {settings.forgetOldest && (
        <div className="-mt-2 flex items-center justify-between gap-3 border-l-2 pl-3">
          <label htmlFor="summarize-forgotten" className="flex flex-col">
            <span className="text-xs font-medium text-muted-foreground">
              {t("summarizeForgotten")}
            </span>
            <span className="text-[11px] text-muted-foreground">{t("summarizeForgottenHint")}</span>
          </label>
          <Switch
            id="summarize-forgotten"
            checked={settings.summarizeForgotten}
            onCheckedChange={(summarizeForgotten) => setSettings({ summarizeForgotten })}
          />
        </div>
      )}
    </>
  );
}
