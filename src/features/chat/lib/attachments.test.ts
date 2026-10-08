import { describe, expect, it } from "vitest";
import { bytesToBase64, encodeWav } from "./attachments";

describe("encodeWav", () => {
  it("writes a 16-bit mono PCM header", () => {
    const wav = encodeWav(new Float32Array([0, 1, -1]), 16_000);
    const view = new DataView(wav.buffer);
    const text = (offset: number) => String.fromCharCode(...wav.subarray(offset, offset + 4));
    expect(text(0)).toBe("RIFF");
    expect(text(8)).toBe("WAVE");
    expect(view.getUint16(22, true)).toBe(1);
    expect(view.getUint32(24, true)).toBe(16_000);
    expect(view.getUint32(40, true)).toBe(6);
    expect(wav.length).toBe(50);
  });

  it("clips and scales samples", () => {
    const wav = encodeWav(new Float32Array([2, -2, 0.5]), 8_000);
    const view = new DataView(wav.buffer);
    expect(view.getInt16(44, true)).toBe(0x7fff);
    expect(view.getInt16(46, true)).toBe(-0x8000);
    expect(view.getInt16(48, true)).toBe(Math.trunc(0.5 * 0x7fff));
  });
});

describe("bytesToBase64", () => {
  it("matches btoa", () => {
    expect(bytesToBase64(new TextEncoder().encode("Calcine"))).toBe(btoa("Calcine"));
  });
});
