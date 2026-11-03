# Changelog

## 1.0.1 (2026-11-03)

### Changes

- Calcine now installs GenieX v0.10.0 on first launch (QAIRT 2.50). Tool calls are now handled
  by GenieX itself, and `stop` sequences are applied by GenieX (d0d77a1)
- When a model fails to load, the chat suggests restarting the PC if it happens right after a
  GenieX update: the NPU can keep the previous version's state until then (d0d77a1)

### Fixes

- Forced tool calls (`tool_choice` naming a function) work again with thinking models on
  GenieX 0.10, which sometimes write only the arguments or leave the reply in the reasoning
  (bcc8a85)

## 1.0.0 (2026-10-09)

The first stable release: a desktop app and a secure, OpenAI-compatible local API for
Qualcomm GenieX on Snapdragon, with the NPU first.

### Models and chat

- Manage GenieX models from the app: Qualcomm AI Hub catalog for your chipset, Hugging Face
  search with precisions, sizes and memory checks, links from ModelScope and Docker Hub, local
  import, remove and clean (9155f29, 9f13225, 57e37c3)
- Streaming chat with reasoning, sampling presets, advanced options, images, voice, model
  preloading and Markdown export (0a1f0d8, 8dd9155, b007ce3, 666e228, 732574c)
- Structured output (`response_format`) and tool calls (`tools`, `tool_choice`) on every
  model, the NPU included: Calcine checks each reply and has the model fix it (60f1198, 4060676)
- Long conversations on the NPU: context counted with each model's tokenizer, the oldest
  messages forgotten in steps that keep GenieX's cache useful, and summarized (c72d6d9, 527801d)
- Speculative decoding in the Chat and the Benchmark (cd33afa)

### API

- Secured gateway on `127.0.0.1:18181`: per-app keys, Host and Origin checks, body checks, a
  request queue and a log without prompts (0a1f0d8)
- Ollama-compatible API, optionally on port 11434 (c07b3a3)
- Other devices on the local network, over HTTPS with network keys and private addresses only,
  off by default (a2ce233)
- `calcine-cli`: the same API without the window, on the `PATH` (b43a42d, 8be7a41)
- Setup snippets for LangChain, Continue and Open WebUI (914e8ea)

### Hardware

- Live NPU, GPU, CPU and memory load, and a self-test per compute unit (9f13225)
- Benchmark page with Qualcomm's `geniex-bench` (44b0746)
- Live power and energy per token on Snapdragon X (878b22a)

### Install and updates

- GenieX downloaded from Qualcomm on first launch and checked against a pinned SHA-256; update,
  roll back and repair it from the app (fe27313, acb05b2)
- Signed Calcine updates on Stable and Beta channels, start with the session, configurable API
  port and allowed web origins (6223b34)
- Linux on ARM64, as a preview: a `.deb` package, GenieX's Linux archive, `calcine-cli geniex`
  for machines without a screen (c9f88cc)
- English and French, light and dark themes, command palette and tray (44a3a1f, 9f13225)

### Fixes

- The network port is free again before it listens anew after a settings change (eb8f0af)
- The chat header stays inside a narrow chat column (71fcc06)
