# Architecture

```
 Other apps ──HTTP──┐                       ┌── React UI (WebView2)
 (OpenAI SDKs…)     │                       │     │ typed commands (tauri-specta)
                    ▼                       │     ▼
           ┌─────────────────┐     ┌──────────────────────┐
           │ calcine-gateway │◄────│ src-tauri (shell)    │  the Chat page streams
           │ 127.0.0.1:18181 │ HTTP│ thin commands, tray  │  through the gateway too
           └────────┬────────┘     └──────────┬───────────┘
                    └──────────┬──────────────┘
                               ▼
                   ┌────────────────────────┐
                   │ calcine-core::Services │  single source of truth
                   └───────────┬────────────┘
                               ▼ trait implementations
               ┌───────────────┼────────────────┬───────────────┐
               ▼               ▼                ▼               ▼
        calcine-geniex    calcine-hub       calcine-hw      calcine-mock
        spawns `geniex …`, Hugging Face,    CPU/RAM/disk,   fake data and a fake
        supervises         AI Hub chipsets  NPU/GPU load    OpenAI server (any OS, CI)
        `geniex serve`     (HTTPS)
```

Security of the gateway is described in [security-model.md](security-model.md).

## Crates

| Crate | Role | Depends on Tauri? |
|---|---|---|
| `crates/calcine-core` | Domain types, model references (pasted names and links), service traits (`ModelStore`, `ModelCatalog`, `RuntimeManager`, `HardwareProbe`), the `JobManager` for long-running work, errors, `Services` | No |
| `crates/calcine-geniex` | GenieX CLI adapter: discovery, command runner (`platform.rs`: what differs on Linux, `LD_LIBRARY_PATH`, parent-death signal, archive install), `pull` with live progress, `geniex serve` supervisor, output parsers tested on real captures (`tests/fixtures/`), installing and updating GenieX (`update/`: release index, verified download, signature, silent installer, cache for rolling back), context windows and token counts with each model's `tokenizer.json` (`context.rs`) | No |
| `crates/calcine-gateway` | Local HTTP API (axum): Host/Origin/API-key checks, optional local network port (`network/`: HTTPS with rustls and a self-signed certificate, allowed address ranges, key-guessing limiter), body sanitizing, OpenAI-compatible proxy with streaming and a one-at-a-time queue (requests rewritten for model aliases and, on battery, the battery saver's power mode), Ollama's API translated, structured output and tools enforced (`structured/`), context fitting, transcription, request log, usage over time (`usage/`: hourly counts per app and model with the energy meter read around each request, saved for 90 days, summaries for the Activity page, `/metrics` in Prometheus' format), `/calcine/v1` management API | No |
| `crates/calcine-docs` | Documents attached in the chat: text of PDF (`pdf-extract`, `lopdf` as a fallback), Word, PowerPoint, OpenDocument, HTML and text files; passages with their page; keyword ranking (BM25, French and English); what fits in a token budget (whole documents or matching passages); the text kept on disk. The app reads files in a child process (`src-tauri/src/documents/`), as PDF readers can panic | No |
| `crates/calcine-hub` | Hub lookups over HTTPS: Hugging Face search, precisions with sizes (named like GenieX), AI Hub chipsets (`ModelDirectory`) | No |
| `crates/calcine-hw` | Hardware probe: CPU, memory and disk (sysinfo), NPU/GPU and drivers (Windows WMI; on Linux the FastRPC device, the `msm`/KGSL GPU and the device tree's SoC), live NPU/GPU load (performance counters), power and energy from the SoC's energy metering (`Energy Meter` counters, found by their English names through PerfLib), battery or mains (WMI `Win32_Battery`, `/sys/class/power_supply`, read at most every 10 s) | No |
| `crates/calcine-mock` | In-memory backend with simulated downloads, for UI work and tests | No |
| `src-tauri/src/cli.rs`, `src-tauri/src/bin/calcine-cli.rs` | `calcine-cli`: a console program (the app is a windowed one) built from the same crate: `serve` runs the same services and gateway as the window (`setup::build`), `keys` manages API keys | Tauri runtime only |
| `src-tauri` | Desktop shell: builds `Services`, exposes commands, exports TypeScript bindings | Yes |

Backend selection: `CALCINE_BACKEND=mock` uses the mock, anything else uses GenieX.

### Inside the crates

```
calcine-core/src/
├─ models/     types.rs · reference.rs (pasted names/links) · store.rs (ModelStore, ModelCatalog)
├─ runtime/    types.rs · manager.rs (RuntimeManager) · server.rs (InferenceServer)
├─ hardware/   types.rs (incl. PowerSource) · probe.rs (HardwareProbe)
├─ jobs/       types.rs · manager.rs (JobManager, JobCtx)
├─ services.rs Services: the set of trait objects every front door uses
└─ error.rs

calcine-geniex/src/
├─ cli/        discovery.rs (find geniex.exe) · runner.rs (spawn with fixed flags)
├─ backend/    models.rs · pull.rs · runtime.rs · serve.rs  (trait implementations)
└─ parse/      one parser per command output, tested on tests/fixtures/

calcine-gateway/src/
├─ security/   mod.rs (Host, Origin, auth middleware) · sanitize.rs (file paths, URLs)
├─ routes/     openai.rs (/v1) · ollama.rs (/api) · manage.rs (/calcine/v1) · metrics.rs
├─ proxy.rs    forwarding and streaming to geniex serve, the request rewritten first
├─ aliases.rs  model aliases · battery.rs (battery saver's power mode)
├─ ollama.rs   Ollama ⇄ OpenAI translation · structured/ (response_format, tools)
├─ context.rs  forget what doesn't fit · transcribe.rs · audio.rs (speech to text)
├─ network/    HTTPS port: tls.rs · allow.rs (address ranges) · limiter.rs
├─ keys.rs     API keys (hashed, scoped) · caller.rs (who is calling)
├─ usage/      bucket.rs · store.rs · recorder.rs · summary.rs · prometheus.rs
└─ log.rs · settings.rs · state.rs · error.rs

calcine-docs/src/   kind.rs (extensions) · error.rs
                    extract/ pdf.rs · html.rs · encoding.rs · office/ (rules.rs, xml.rs)
                    passages/ clean.rs · paragraphs.rs · pack.rs
                    search/ terms.rs · stopwords.rs · bm25.rs
                    select/ whole.rs · passages.rs · spread.rs · format.rs
                    store/ info.rs

calcine-hub/src/    hf.rs (Hugging Face payloads) · quant.rs (precision names) · aihub.rs (chipsets)

calcine-hw/src/     snapshot.rs · usage.rs (live load) · power.rs (power, energy) · system.rs (sysinfo)
                    battery.rs (cached) · linux.rs
                    windows/ devices.rs (WMI) · battery.rs (WMI) · energy.rs · gpu_engines.rs · pdh.rs (PDH, the only `unsafe` code)
calcine-mock/src/   data.rs (sample data) · models.rs · directory.rs · runtime.rs · hardware.rs · server.rs

src-tauri/
├─ src/ipc/      mod.rs (bindings builder) · commands/ (one file per domain) · events.rs · error.rs
├─ src/setup/    services.rs (pick the backend) · gateway.rs · start and shutdown
├─ src/desktop/  tray.rs · window behaviour · quick_ask.rs (global shortcut, quick question window)
│                notifications.rs · locale.rs · process.rs (Job Object)
├─ windows/      license-and-privacy.txt (installer license page) · test-manifest.xml
└─ tests/        tests that link Tauri (bindings export)
```

Rule of thumb: one folder per domain, data types in `types.rs`, the service
trait next to them, and one implementation file per trait in each backend.

## Jobs

Downloads and imports (and later updates and benchmarks) run as jobs:
`Services::start_pull` spawns one through the `JobManager`, which tracks its
state and progress, supports cancellation, and broadcasts every update. The
Tauri shell forwards updates to the webview as the typed `JobUpdated` event
(throttled to ~5 per second per job); the UI mirrors them into the
TanStack Query cache (`src/features/tasks/api.ts`).

## Desktop behavior

- Closing the window hides it to the tray; quit from the tray menu.
- Launching Calcine again focuses the running instance (single-instance plugin).
- A finished download or import shows a system notification when Calcine
  isn't in front.
- The tray menu and notifications follow the UI language (`set_language`).
- A global shortcut (Ctrl+Shift+Space by default, Settings › Quick question)
  or the tray menu opens the quick question window, label `quick`: built the
  first time, then shown and hidden; it hides when it loses focus. It loads the
  same page as the main window: `src/main.tsx` renders `QuickAskApp` for that
  label. It reads the chat's settings from local storage but never writes the
  chat's store (two windows writing it would overwrite each other); "Continue
  in Calcine" sends the exchange to the main window as an event
  (`quick-ask:continue`).

## Frontend

```
src/
├─ main.tsx        picks the root: the app, or the quick question window
├─ app/            router, navigation, shell (sidebar, topbar, ⌘K palette, task drawer)
├─ features/<x>/   one folder per feature:
│    ├─ <x>-page.tsx      the route's page
│    ├─ api.ts            TanStack Query options, mutations and hooks
│    ├─ components/       pieces of the page
│    ├─ lib/              pure logic (formatting, reducers) with its tests
│    └─ messages.ts       the feature's strings, English and French
├─ i18n/          translations: defineMessages, useT, language store, shared words
├─ components/ui/  shadcn/ui-style primitives, themed with Ember tokens
├─ components/calcine/  app-specific building blocks:
│    brand/ (logo) · layout/ (page frame) · feedback/ (empty, error, confirm,
│    status dot) · badges/ (runtime, model type)
├─ hooks/          generic React hooks
├─ stores/         Zustand stores (UI state, persisted theme)
├─ styles/         globals.css: Ember design tokens (dark default, light variant)
└─ lib/            bindings.ts (generated), api.ts (unwrap, errors), utils
```

## Adding a feature

1. Add the domain method to a trait in `calcine-core` and implement it in
   `calcine-geniex` and `calcine-mock` (with a parser test on real output).
2. Add a thin command in `src-tauri/src/ipc/commands/` and register it in
   `src-tauri/src/ipc/mod.rs`.
3. Run `cargo test -p calcine` to regenerate `src/lib/bindings.ts` (CI fails if
   it's stale).
4. Build the UI in `src/features/<feature>/` and, for a new page, add its route
   in `src/app/router.tsx` and its entry in `src/app/navigation.ts`.

## Tests that link Tauri

On Windows, only integration test binaries get the app manifest Tauri needs
(see `src-tauri/build.rs`). Put tests that link Tauri in `src-tauri/tests/`,
not in `src-tauri/src/`.

## Translations

The UI speaks English and French. Each feature keeps its strings in
`messages.ts`, declared with `defineMessages({ en, fr })`: TypeScript fails if
French misses a key. Components call `const t = useT(messages)`, then
`t("key", { name })`, `t.plural("key", count)` (`key_one` / `key_other`) or
`t.rich("key", { code: (text) => <code>{text}</code> })`. Shared words live in
`src/i18n/common.ts`. Byte sizes and dates follow the language
(`src/lib/format.ts`). The language follows Windows by default and can be set
in Settings.
