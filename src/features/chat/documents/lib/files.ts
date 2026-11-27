/** File names and kinds, from paths on Windows or Linux. */

/** Image files vision models take, as attachments. */
const IMAGE_EXTENSIONS = ["png", "jpg", "jpeg", "webp", "gif", "bmp"];

export function fileName(path: string): string {
  return path.split(/[\\/]/).at(-1) ?? path;
}

export function extensionOf(path: string): string {
  const name = fileName(path);
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
}

export function isImagePath(path: string): boolean {
  return IMAGE_EXTENSIONS.includes(extensionOf(path));
}
