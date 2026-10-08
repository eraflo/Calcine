/**
 * A library name for a model imported from `path`: `local/<folder or zip
 * name>`, cleaned to what GenieX accepts (no spaces, no `-GGUF` suffix
 * needed).
 */
export function suggestImportName(path: string): string {
  const base =
    path
      .replace(/[\\/]+$/, "")
      .split(/[\\/]/)
      .pop() ?? "";
  const stem = base.replace(/\.zip$/i, "");
  const cleaned = stem
    .trim()
    .replace(/\s+/g, "-")
    .replace(/[^\w.-]/g, "")
    .replace(/-+/g, "-")
    .replace(/^[-.]+|[-.]+$/g, "");
  return `local/${cleaned || "model"}`;
}

/** Whether `name` has the `owner/model` shape GenieX expects. */
export function isValidImportName(name: string): boolean {
  return /^[\w.-]+\/[\w.-]+$/.test(name.trim());
}
