import { describe, expect, it } from "vitest";
import { isValidImportName, suggestImportName } from "./import-name";

describe("suggestImportName", () => {
  it("uses the folder or archive name", () => {
    expect(suggestImportName("C:\\Models\\Qwen3-0.6B-GGUF")).toBe("local/Qwen3-0.6B-GGUF");
    expect(suggestImportName("C:\\Downloads\\qwen3_4b-genie.zip")).toBe("local/qwen3_4b-genie");
    expect(suggestImportName("/home/me/models/llama/")).toBe("local/llama");
  });

  it("removes spaces and odd characters", () => {
    expect(suggestImportName("D:\\My Models (copy)")).toBe("local/My-Models-copy");
    expect(suggestImportName("C:\\")).toBe("local/C");
  });
});

describe("isValidImportName", () => {
  it("wants owner/model", () => {
    expect(isValidImportName("local/qwen3")).toBe(true);
    expect(isValidImportName("qwen3")).toBe(false);
    expect(isValidImportName("local/my model")).toBe(false);
  });
});
