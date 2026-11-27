import { describe, expect, it } from "vitest";
import { withMath } from "./math";

describe("withMath", () => {
  it("turns LaTeX delimiters into remark-math's", () => {
    expect(withMath("Area: \\(\\pi r^2\\).")).toBe("Area: $$\\pi r^2$$.");
    expect(withMath("So\n\\[ E = mc^2 \\]\ndone")).toBe("So\n\n$$\nE = mc^2\n$$\n\ndone");
  });

  it("keeps single-dollar formulas inline", () => {
    expect(withMath("with $x^2$ and $\\alpha$")).toBe("with $$x^2$$ and $$\\alpha$$");
  });

  it("leaves prices alone", () => {
    expect(withMath("It costs $5 and $10.")).toBe("It costs $5 and $10.");
    expect(withMath("Between $ 3 and $ 4")).toBe("Between $ 3 and $ 4");
  });

  it("leaves code alone", () => {
    const code = "```sh\necho $HOME $PATH\n```\nand `$x$` inline";
    expect(withMath(code)).toBe(code);
  });

  it("leaves display math as it is", () => {
    expect(withMath("$$\nx = 1\n$$")).toBe("$$\nx = 1\n$$");
  });
});
