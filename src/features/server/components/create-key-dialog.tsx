import { TriangleAlert } from "lucide-react";
import { type FormEvent, useState } from "react";
import { CopyButton } from "@/components/calcine/actions/copy-button";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { Field, Input } from "@/components/ui/field";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import type { CreatedApiKey, KeyScope } from "@/lib/api";
import { useCreateApiKey } from "../api";
import { messages } from "../messages";

type MessageKey = keyof (typeof messages)["en"];

const PERMISSIONS: { scope: KeyScope; label: MessageKey; hint: MessageKey }[] = [
  { scope: "inference", label: "runModels", hint: "runModelsHint" },
  { scope: "manage", label: "manageModels", hint: "manageModelsHint" },
];

export function CreateKeyDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const t = useT(messages);
  const tc = useT(common);
  const create = useCreateApiKey();
  const [name, setName] = useState("");
  const [scopes, setScopes] = useState<KeyScope[]>(["inference"]);
  const [allowLocalFiles, setAllowLocalFiles] = useState(false);
  const [created, setCreated] = useState<CreatedApiKey | null>(null);

  const close = (next: boolean) => {
    onOpenChange(next);
    if (!next) {
      setName("");
      setScopes(["inference"]);
      setAllowLocalFiles(false);
      setCreated(null);
      create.reset();
    }
  };

  const submit = (event: FormEvent) => {
    event.preventDefault();
    create.mutate({ name, scopes, allowLocalFiles }, { onSuccess: setCreated });
  };

  const toggle = (scope: KeyScope, on: boolean) =>
    setScopes((current) => (on ? [...current, scope] : current.filter((s) => s !== scope)));

  return (
    <Dialog open={open} onOpenChange={close}>
      <DialogContent className="top-1/2 max-w-md -translate-y-1/2 p-5">
        {created ? (
          <div className="flex flex-col gap-4">
            <DialogTitle className="text-base font-semibold">
              {t("keyFor", { name: created.key.name })}
            </DialogTitle>
            <DialogDescription className="text-sm text-muted-foreground">
              {t("keyCopyNow")}
            </DialogDescription>
            <div className="flex items-center gap-2 rounded-md border bg-background px-3 py-2">
              <code className="min-w-0 flex-1 font-mono text-[12px] break-all">
                {created.token}
              </code>
              <CopyButton value={created.token} />
            </div>
            <div className="flex justify-end">
              <Button variant="default" onClick={() => close(false)}>
                {t("done")}
              </Button>
            </div>
          </div>
        ) : (
          <form onSubmit={submit} className="flex flex-col gap-4">
            <DialogTitle className="text-base font-semibold">{t("newApiKey")}</DialogTitle>
            <DialogDescription className="sr-only">{t("newApiKeyHint")}</DialogDescription>
            <Field label={t("appName")} htmlFor="key-name">
              <Input
                id="key-name"
                value={name}
                onChange={(event) => setName(event.target.value)}
                placeholder="Open WebUI"
                autoFocus
              />
            </Field>
            <fieldset className="flex flex-col gap-2">
              <legend className="mb-1 text-xs font-medium text-muted-foreground">
                {t("permissions")}
              </legend>
              {PERMISSIONS.map(({ scope, label, hint }) => (
                <label key={scope} className="flex cursor-pointer items-start gap-2.5 text-sm">
                  <input
                    type="checkbox"
                    checked={scopes.includes(scope)}
                    onChange={(event) => toggle(scope, event.target.checked)}
                    className="mt-0.5 accent-[var(--primary)]"
                  />
                  <span>
                    {t(label)}
                    <span className="block text-xs text-muted-foreground">{t(hint)}</span>
                  </span>
                </label>
              ))}
              <label className="flex cursor-pointer items-start gap-2.5 text-sm">
                <input
                  type="checkbox"
                  checked={allowLocalFiles}
                  onChange={(event) => setAllowLocalFiles(event.target.checked)}
                  className="mt-0.5 accent-[var(--primary)]"
                />
                <span>
                  {t("localFilesLabel")}
                  <span className="flex items-start gap-1 text-xs text-warning">
                    <TriangleAlert className="mt-px size-3.5 shrink-0" />
                    {t("localFilesWarning")}
                  </span>
                </span>
              </label>
            </fieldset>
            {create.isError && <p className="text-xs text-destructive">{create.error.message}</p>}
            <div className="flex justify-end gap-2">
              <Button type="button" variant="ghost" onClick={() => close(false)}>
                {tc("cancel")}
              </Button>
              <Button type="submit" variant="default" disabled={create.isPending}>
                {t("createKey")}
              </Button>
            </div>
          </form>
        )}
      </DialogContent>
    </Dialog>
  );
}
