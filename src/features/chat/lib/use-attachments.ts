import { useCallback, useState } from "react";
import { commands, type DocumentProblem, unwrap } from "@/lib/api";
import { newId } from "../store";
import type { PendingAttachment } from "../use-send";
import { prepareImage } from "./attachments";
import { type DocumentInfo, fileName, isImagePath } from "./documents";

/** A document being read, read, or that couldn't be. */
export type PendingDocument =
  | { key: string; name: string; status: "reading" }
  | { key: string; name: string; status: "ready"; document: DocumentInfo }
  | { key: string; name: string; status: "failed"; problem: DocumentProblem };

export type AttachmentProblem =
  | { kind: "image-needs-vision"; name: string }
  | { kind: "image-failed"; name: string; message: string };

/**
 * Files attached to the next message: images (for vision models) and
 * documents (for every model), from the file picker, a drop or a paste.
 * Documents are read as soon as they're added.
 */
export function useAttachments({
  media,
  onProblem,
}: {
  /** The model takes images. */
  media: boolean;
  onProblem: (problem: AttachmentProblem) => void;
}) {
  const [images, setImages] = useState<PendingAttachment[]>([]);
  const [documents, setDocuments] = useState<PendingDocument[]>([]);

  const addImage = useCallback(async (name: string, blob: Blob) => {
    const { dataUrl, thumbnail } = await prepareImage(blob);
    setImages((current) => [
      ...current,
      {
        attachment: { id: newId(), kind: "image", name, thumbnail },
        payload: { kind: "image", dataUrl },
      },
    ]);
  }, []);

  const addDocument = useCallback(async (path: string) => {
    const key = newId();
    const name = fileName(path);
    setDocuments((current) => [...current, { key, name, status: "reading" }]);
    const settle = (next: PendingDocument) =>
      setDocuments((current) => current.map((item) => (item.key === key ? next : item)));
    try {
      const imported = await unwrap(() => commands.importDocument(path));
      settle(
        imported.status === "ready"
          ? { key, name, status: "ready", document: imported.document }
          : { key, name, status: "failed", problem: imported.problem },
      );
    } catch {
      settle({ key, name, status: "failed", problem: "unreadable" });
    }
  }, []);

  /** Files picked or dropped, by path. */
  const addPaths = useCallback(
    async (paths: readonly string[]) => {
      await Promise.all(
        paths.map(async (path) => {
          if (!isImagePath(path)) return addDocument(path);
          const name = fileName(path);
          if (!media) {
            onProblem({ kind: "image-needs-vision", name });
            return;
          }
          try {
            const dataUrl = await unwrap(() => commands.readImage(path));
            await addImage(name, await (await fetch(dataUrl)).blob());
          } catch (error) {
            onProblem({ kind: "image-failed", name, message: describe(error) });
          }
        }),
      );
    },
    [media, addDocument, addImage, onProblem],
  );

  /** Images pasted in the message box. */
  const addImageFiles = useCallback(
    async (files: Iterable<File>) => {
      for (const file of files) {
        if (!file.type.startsWith("image/")) continue;
        try {
          await addImage(file.name || "image", file);
        } catch (error) {
          onProblem({ kind: "image-failed", name: file.name, message: describe(error) });
        }
      }
    },
    [addImage, onProblem],
  );

  const pick = useCallback(async () => {
    const paths = await commands.pickAttachments(media);
    await addPaths(paths);
  }, [media, addPaths]);

  const removeImage = useCallback((id: string) => {
    setImages((current) => current.filter((item) => item.attachment.id !== id));
  }, []);

  /** Remove a document before sending; its text is deleted. */
  const removeDocument = useCallback((key: string) => {
    setDocuments((current) => {
      const removed = current.find((item) => item.key === key);
      if (removed?.status === "ready") void commands.removeDocuments([removed.document.id]);
      return current.filter((item) => item.key !== key);
    });
  }, []);

  /** Images go when the model can't take them. */
  const dropImages = useCallback(() => setImages([]), []);

  const clear = useCallback(() => {
    setImages([]);
    setDocuments([]);
  }, []);

  const ready = documents.flatMap((item) => (item.status === "ready" ? [item.document] : []));
  return {
    images,
    documents,
    /** Documents read and ready to send. */
    ready,
    /** Some documents are still being read. */
    reading: documents.some((item) => item.status === "reading"),
    pick,
    addPaths,
    addImageFiles,
    removeImage,
    removeDocument,
    dropImages,
    clear,
  };
}

export type Attachments = ReturnType<typeof useAttachments>;

function describe(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
