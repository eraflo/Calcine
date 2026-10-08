# Kiln

Desktop GUI and local API gateway for [Qualcomm GenieX](https://github.com/qualcomm/GenieX) —
run LLMs and VLMs on the Snapdragon NPU / GPU / CPU and expose them to any app through an
OpenAI-compatible endpoint on `localhost`.

> Status: early development.

## Requirements

- Snapdragon device (Windows 11 ARM64; Linux ARM64 planned)
- [GenieX CLI](https://github.com/qualcomm/GenieX/releases) installed
- Rust (stable, `aarch64-pc-windows-msvc`), Node.js ≥ 22, [bun](https://bun.sh)

## Stack

Tauri v2 · Rust (tokio, axum) · React + TypeScript + Vite
