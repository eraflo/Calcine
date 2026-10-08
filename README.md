# Calcine

Desktop app and local API gateway for [Qualcomm GenieX](https://github.com/qualcomm/GenieX):
download, manage and run LLMs and VLMs on the Snapdragon NPU / GPU / CPU, and
expose them to any app through a secured, OpenAI-compatible endpoint on
`127.0.0.1:18181`.

> Status: beta. Download the latest installer from
> [Releases](https://github.com/eraflo/Calcine/releases) (Windows 11 ARM64).
> Builds aren't code-signed yet, so SmartScreen warns before installing.

## Features

- Every GenieX command in the UI: pull, import, remove, clean, list, catalog, set-type, serve, config, update
- Discover models on Qualcomm AI Hub and Hugging Face, with precisions, sizes and memory/disk checks
- Chat with every inference option, adapted to the model's runtime, with images and voice for vision models
- Live hardware view (Hexagon NPU, Adreno GPU, Oryon CPU, memory) and a per-unit speed self-test
- Local OpenAI-compatible API with per-app keys, origin checks and a request log
- GenieX bundled in the installer, updatable (or rolled back) from the app; Calcine updates itself
- English and French

## Requirements

- Snapdragon device running Windows 11 ARM64 (Linux ARM64 planned)
- GenieX is installed automatically by the Calcine installer

## Development

Tauri v2 · Rust · React + TypeScript + Vite · Tailwind CSS · bun.

```bash
bun install
bun run app:mock   # runs anywhere, with fake data
bun run app        # on a Snapdragon device with GenieX installed
```

See [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/architecture.md](docs/architecture.md).

## License

[MIT](LICENSE). Calcine redistributes GenieX under its BSD 3-Clause license — see
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). Calcine is an independent project,
not affiliated with Qualcomm.
