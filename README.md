# Calcine

Desktop app and local API gateway for [Qualcomm GenieX](https://github.com/qualcomm/GenieX):
download, manage and run LLMs and VLMs on the Snapdragon NPU / GPU / CPU, and
expose them to any app through a secured, OpenAI-compatible endpoint on
`127.0.0.1:18181`.

> Status: early development — not yet released.

## Features (planned)

- One-click access to every GenieX command: pull, remove, list, catalog, serve, config, update
- Model library and discovery (Qualcomm AI Hub, Hugging Face, ModelScope, Docker Hub, local import)
- Chat playground with every inference option, adapted to the model's runtime
- Live hardware view: Hexagon NPU, Adreno GPU, Oryon CPU, memory
- Local API with per-app keys, origin checks and request log
- GenieX bundled in the installer and updatable from the app

## Requirements

- Snapdragon device running Windows 11 ARM64 (Linux ARM64 planned)
- GenieX is installed automatically by the Calcine installer

## Development

Tauri v2 · Rust (tokio, axum) · React + TypeScript + Vite · bun.
See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE). Calcine redistributes GenieX under its BSD 3-Clause license — see
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). Calcine is an independent project,
not affiliated with Qualcomm.
