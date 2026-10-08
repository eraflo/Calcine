import { FileArchive, FolderInput, FolderOpen } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "@/components/ui/dialog";
import { Field, Input } from "@/components/ui/field";
import { SegmentedControl } from "@/components/ui/segmented-control";
import { useImportModel } from "@/features/tasks/api";
import { useT } from "@/i18n";
import { common } from "@/i18n/common";
import type { ModelType } from "@/lib/api";
import { pickImportSource } from "../api";
import { isValidImportName, suggestImportName } from "../lib/import-name";
import { messages } from "../messages";

type TypeChoice = "auto" | Exclude<ModelType, "unknown">;

/** Import a model folder or AI Hub `.zip` from this PC (`geniex pull --model-hub localfs`). */
export function ImportDialog({
  open,
  initialPath,
  onOpenChange,
}: {
  open: boolean;
  /** Prefilled from a drag and drop. */
  initialPath: string | null;
  onOpenChange: (open: boolean) => void;
}) {
  const t = useT(messages);
  const tc = useT(common);
  const importModel = useImportModel();
  const resetImport = importModel.reset;
  const [path, setPath] = useState("");
  const [name, setName] = useState("");
  const [nameEdited, setNameEdited] = useState(false);
  const [type, setType] = useState<TypeChoice>("auto");

  // Start fresh each time the dialog opens.
  useEffect(() => {
    if (!open) return;
    setPath(initialPath ?? "");
    setName(initialPath ? suggestImportName(initialPath) : "");
    setNameEdited(false);
    setType("auto");
    resetImport();
  }, [open, initialPath, resetImport]);

  const choose = async (source: "folder" | "archive") => {
    const picked = await pickImportSource(source);
    if (!picked) return;
    setPath(picked);
    if (!nameEdited) setName(suggestImportName(picked));
  };

  const validName = isValidImportName(name);
  const submit = () =>
    importModel.mutate(
      { name: name.trim(), path, modelType: type === "auto" ? null : type },
      { onSuccess: () => onOpenChange(false) },
    );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="top-1/2 max-w-xl -translate-y-1/2">
        <div className="flex flex-col gap-1 border-b px-5 py-4">
          <DialogTitle className="text-base font-semibold">{t("importTitle")}</DialogTitle>
          <DialogDescription className="text-xs text-muted-foreground">
            {t("importDescription")}
          </DialogDescription>
        </div>

        <div className="flex flex-col gap-5 px-5 py-4">
          <Field label={t("importSource")} hint={t("importSourceHint")}>
            <div className="flex flex-col gap-2">
              <div className="flex gap-2">
                <Button variant="secondary" size="sm" onClick={() => choose("folder")}>
                  <FolderOpen />
                  {t("importFolder")}
                </Button>
                <Button variant="secondary" size="sm" onClick={() => choose("archive")}>
                  <FileArchive />
                  {t("importZip")}
                </Button>
              </div>
              {path && (
                <p
                  className="truncate rounded-md border bg-background px-3 py-1.5 font-mono text-xs"
                  title={path}
                >
                  {path}
                </p>
              )}
            </div>
          </Field>

          <Field label={t("importName")} htmlFor="import-name" hint={t("importNameHint")}>
            <Input
              id="import-name"
              value={name}
              onChange={(event) => {
                setName(event.target.value);
                setNameEdited(true);
              }}
              placeholder="local/my-model"
              spellCheck={false}
              aria-invalid={name.length > 0 && !validName}
              className="font-mono"
            />
          </Field>

          <Field label={t("typeLabel")} hint={t("typeHint")}>
            <SegmentedControl
              name="import-type"
              label={t("typeLabel")}
              value={type}
              onChange={setType}
              className="self-start"
              options={[
                { value: "auto", label: t("typeAuto") },
                { value: "llm", label: t("typeText") },
                { value: "vlm", label: t("typeVision") },
              ]}
            />
          </Field>
        </div>

        <div className="flex items-center gap-2 border-t px-5 py-3">
          <p className="min-w-0 flex-1 truncate text-xs text-destructive">
            {importModel.isError ? importModel.error.message : null}
          </p>
          <DialogClose asChild>
            <Button variant="ghost">{tc("cancel")}</Button>
          </DialogClose>
          <Button onClick={submit} disabled={!path || !validName || importModel.isPending}>
            <FolderInput />
            {t("importStart")}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
