import { describe, expect, it } from "vitest";
import { extensionOf, fileName, isImagePath } from "./files";

describe("files", () => {
  it("reads file names and extensions from Windows and Unix paths", () => {
    expect(fileName("C:\\Users\\me\\Report 2027.PDF")).toBe("Report 2027.PDF");
    expect(extensionOf("C:\\Users\\me\\Report 2027.PDF")).toBe("pdf");
    expect(extensionOf("/home/me/.bashrc")).toBe("");
    expect(isImagePath("/tmp/photo.JPEG")).toBe(true);
    expect(isImagePath("/tmp/notes.md")).toBe(false);
  });
});
