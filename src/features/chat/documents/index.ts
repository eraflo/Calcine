/**
 * Documents attached to a conversation. Their text stays in Calcine's data
 * folder (read by the backend into passages); the chat keeps their
 * description, and asks before each message what the model should read.
 *
 * - `lib/`: request parts, token budget, sources, file names, the
 *   attachments hook
 * - `components/`: chips, the documents panel, the drop zone, sources
 */

export type { ContextMode, DocumentContext, DocumentInfo, Source } from "@/lib/api";
export { DocumentChip } from "./components/document-chip";
export { DocumentsButton } from "./components/documents-button";
export { DropZone } from "./components/drop-zone";
export { Sources } from "./components/sources";
export { documentBudget } from "./lib/budget";
export { documentParts, documentQueries, withDocuments, withPassages } from "./lib/prompt";
export { groupSources, newDocuments } from "./lib/sources";
export {
  type AttachmentProblem,
  type Attachments,
  type PendingDocument,
  useAttachments,
} from "./lib/use-attachments";
export { documentMessages } from "./messages";
