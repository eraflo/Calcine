/**
 * Images and recordings sent to vision models, prepared in the webview:
 * images are downscaled to JPEG, audio is resampled to 16 kHz mono WAV, the
 * formats llama.cpp's multimodal projectors expect.
 */

export type Attachment = {
  id: string;
  kind: "image" | "audio";
  name: string;
  /** Small JPEG preview, kept with the conversation. */
  thumbnail?: string;
  /** Length of a recording. */
  durationSeconds?: number;
};

/** Longest image side sent to the model. */
const MAX_IMAGE_SIDE = 1280;
const THUMBNAIL_SIDE = 160;
/** Audio models are trained on 16 kHz speech. */
const AUDIO_SAMPLE_RATE = 16_000;
export const MAX_RECORDING_SECONDS = 60;

export type PreparedImage = { dataUrl: string; thumbnail: string };

/** Decode an image file and produce what the model and the thread need. */
export async function prepareImage(file: Blob): Promise<PreparedImage> {
  const bitmap = await createImageBitmap(file);
  try {
    return {
      dataUrl: drawScaled(bitmap, MAX_IMAGE_SIDE, 0.9),
      thumbnail: drawScaled(bitmap, THUMBNAIL_SIDE, 0.8),
    };
  } finally {
    bitmap.close();
  }
}

function drawScaled(bitmap: ImageBitmap, maxSide: number, quality: number): string {
  const scale = Math.min(1, maxSide / Math.max(bitmap.width, bitmap.height));
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(bitmap.width * scale));
  canvas.height = Math.max(1, Math.round(bitmap.height * scale));
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Canvas isn't available");
  // JPEG has no alpha: paint transparent areas white rather than black.
  context.fillStyle = "#fff";
  context.fillRect(0, 0, canvas.width, canvas.height);
  context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
  return canvas.toDataURL("image/jpeg", quality);
}

/** Turn a browser recording (WebM/Opus) into 16 kHz mono WAV, base64-encoded. */
export async function recordingToWav(
  recording: Blob,
): Promise<{ base64: string; seconds: number }> {
  const decoder = new AudioContext();
  let decoded: AudioBuffer;
  try {
    decoded = await decoder.decodeAudioData(await recording.arrayBuffer());
  } finally {
    void decoder.close();
  }
  const frames = Math.ceil(decoded.duration * AUDIO_SAMPLE_RATE);
  const offline = new OfflineAudioContext(1, Math.max(1, frames), AUDIO_SAMPLE_RATE);
  const source = offline.createBufferSource();
  source.buffer = decoded;
  source.connect(offline.destination);
  source.start();
  const mono = await offline.startRendering();
  return {
    base64: bytesToBase64(encodeWav(mono.getChannelData(0), AUDIO_SAMPLE_RATE)),
    seconds: decoded.duration,
  };
}

/** 16-bit PCM WAV file for mono `samples` in [-1, 1]. */
export function encodeWav(samples: Float32Array, sampleRate: number): Uint8Array {
  const bytes = new Uint8Array(44 + samples.length * 2);
  const view = new DataView(bytes.buffer);
  const text = (offset: number, value: string) => {
    for (let index = 0; index < value.length; index++) {
      view.setUint8(offset + index, value.charCodeAt(index));
    }
  };
  text(0, "RIFF");
  view.setUint32(4, 36 + samples.length * 2, true);
  text(8, "WAVE");
  text(12, "fmt ");
  view.setUint32(16, 16, true); // PCM chunk size
  view.setUint16(20, 1, true); // PCM
  view.setUint16(22, 1, true); // mono
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * 2, true); // bytes per second
  view.setUint16(32, 2, true); // block align
  view.setUint16(34, 16, true); // bits per sample
  text(36, "data");
  view.setUint32(40, samples.length * 2, true);
  for (let index = 0; index < samples.length; index++) {
    const sample = Math.max(-1, Math.min(1, samples[index] ?? 0));
    view.setInt16(44 + index * 2, sample < 0 ? sample * 0x8000 : sample * 0x7fff, true);
  }
  return bytes;
}

export function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunk = 0x8000;
  for (let index = 0; index < bytes.length; index += chunk) {
    binary += String.fromCharCode(...bytes.subarray(index, index + chunk));
  }
  return btoa(binary);
}
