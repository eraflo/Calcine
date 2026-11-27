import { describe, expect, it } from "vitest";
import { axisLabel, labelEvery, pointLabel, utcOffsetMinutes } from "./time";

describe("activity time labels", () => {
  it("names hours, weekdays or dates", () => {
    const tuesday = new Date(2026, 10, 24, 14, 0).getTime();
    expect(axisLabel(tuesday, "day", "en-GB")).toBe("14:00");
    expect(axisLabel(tuesday, "week", "en-GB")).toBe("Tue");
    expect(axisLabel(tuesday, "month", "en-GB")).toBe("24 Nov");
    expect(pointLabel(tuesday, 3_600_000, "day", "en-GB")).toBe("14:00 – 15:00");
    expect(pointLabel(tuesday, 86_400_000, "week", "en-GB")).toBe("Tue 24 Nov");
  });

  it("spaces axis labels by period", () => {
    expect(labelEvery("day")).toBe(6);
    expect(labelEvery("month")).toBe(5);
  });

  it("follows the local time zone", () => {
    const date = new Date();
    expect(utcOffsetMinutes(date)).toBe(-date.getTimezoneOffset());
  });
});
