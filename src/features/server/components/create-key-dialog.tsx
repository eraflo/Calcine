import { TriangleAlert } from "lucide-react";
import { type FormEvent, useState } from "react";
import { CopyButton } from "@/components/calcine/actions/copy-button";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { Field, Input } from "@/components/ui/field";
import type { CreatedApiKey, KeyScope } from "@/lib/api";
import { useCreateApiKey } from "../api";

const PERMISSIONS: { scope: KeyScope; label: string; hint: string }[] = [
  { scope: "inference", label: "Run models", hint: "Chat, completions and the model list." },
  {
    scope: "manage",
    label: "Manage models",
    hint: "Download and remove models, follow downloads.",
  },
];

export function CreateKeyDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
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
              Key for {created.key.name}
            </DialogTitle>
            <DialogDescription className="text-sm text-muted-foreground">
              Copy it now: it won't be shown again. Calcine only keeps a fingerprint.
            </DialogDescription>
            <div className="flex items-center gap-2 rounded-md border bg-background px-3 py-2">
              <code className="min-w-0 flex-1 font-mono text-[12px] break-all">
                {created.token}
              </code>
              <CopyButton value={created.token} />
            </div>
            <div className="flex justify-end">
              <Button variant="default" onClick={() => close(false)}>
                Done
              </Button>
            </div>
          </div>
        ) : (
          <form onSubmit={submit} className="flex flex-col gap-4">
            <DialogTitle className="text-base font-semibold">New API key</DialogTitle>
            <DialogDescription className="sr-only">
              Name the app and choose what it may do.
            </DialogDescription>
            <Field label="App name" htmlFor="key-name">
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
                Permissions
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
                    {label}
                    <span className="block text-xs text-muted-foreground">{hint}</span>
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
                  Read local files and URLs
                  <span className="flex items-start gap-1 text-xs text-warning">
                    <TriangleAlert className="mt-px size-3.5 shrink-0" />
                    Lets the app make GenieX open any file on this PC or fetch any URL. Only for
                    apps you trust; others can still send images inline.
                  </span>
                </span>
              </label>
            </fieldset>
            {create.isError && <p className="text-xs text-destructive">{create.error.message}</p>}
            <div className="flex justify-end gap-2">
              <Button type="button" variant="ghost" onClick={() => close(false)}>
                Cancel
              </Button>
              <Button type="submit" variant="default" disabled={create.isPending}>
                Create key
              </Button>
            </div>
          </form>
        )}
      </DialogContent>
    </Dialog>
  );
}
