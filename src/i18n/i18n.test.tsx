import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { defineMessages, translator } from ".";

const messages = defineMessages({
  en: {
    hello: "Hello {name}",
    models_one: "{count} model",
    models_other: "{count} models",
    hint: "Type <code>owner/model</code> here",
  },
  fr: {
    hello: "Bonjour {name}",
    models_one: "{count} modèle",
    models_other: "{count} modèles",
    hint: "Tapez <code>owner/model</code> ici",
  },
});

describe("translator", () => {
  it("interpolates", () => {
    expect(translator(messages, "en")("hello", { name: "Ada" })).toBe("Hello Ada");
    expect(translator(messages, "fr")("hello", { name: "Ada" })).toBe("Bonjour Ada");
  });

  it("follows each language's plural rules", () => {
    const en = translator(messages, "en");
    const fr = translator(messages, "fr");
    expect(en.plural("models", 0)).toBe("0 models");
    expect(en.plural("models", 1)).toBe("1 model");
    // French uses the singular for 0 and 1.
    expect(fr.plural("models", 0)).toBe("0 modèle");
    expect(fr.plural("models", 1_500)).toBe("1 500 modèles");
  });

  it("renders tags as elements", () => {
    const t = translator(messages, "en");
    render(<p>{t.rich("hint", { code: (text) => <code>{text}</code> })}</p>);
    expect(screen.getByText("owner/model").tagName).toBe("CODE");
    expect(screen.getByText(/Type/).textContent).toBe("Type owner/model here");
  });
});
