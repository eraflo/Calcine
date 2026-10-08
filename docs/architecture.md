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
| `crates/calcine-geniex` | GenieX CLI adapter: discovery, command runner, `pull` with live progress, `geniex serve` supervisor, output parsers tested on real captures (`tests/fixtures/`), installing and updating GenieX (`update/`: release index, verified download, signature, silent installer, cache for rolling back) | No |
| `crates/calcine-gateway` | Local HTTP API (axum): Host/Origin/API-key checks, body sanitizing, OpenAI-compatible proxy with streaming and a one-at-a-time queue, request log, `/calcine/v1` management API | No |
| `crates/calcine-hub` | Hub lookups over HTTPS: Hugging Face search, precisions with sizes (named like GenieX), AI Hub chipsets (`ModelDirectory`) | No |
| `crates/calcine-hw` | Hardware probe: CPU, memory and disk (sysinfo), NPU/GPU and drivers (Windows WMI), live NPU/GPU load (performance counters) | No |
| `crates/calcine-mock` | In-memory backend with simulated downloads, for UI work and tests | No |
| `src-tauri` | Desktop shell: builds `Services`, exposes commands, exports TypeScript bindings | Yes |

Backend selection: `CALCINE_BACKEND=mock` uses the mock, anything else uses GenieX.

### Inside the crates

```
calcine-core/src/
├─ models/     types.rs · reference.rs (pasted names/links) · store.rs (ModelStore, ModelCatalog)
├─ runtime/    types.rs · manager.rs (RuntimeManager) · server.rs (InferenceServer)
├─ hardware/   types.rs · probe.rs (HardwareProbe)
├─ jobs/       types.rs · manager.rs (JobManager, JobCtx)
├─ services.rs Services: the set of trait objects every front door uses
└─ error.rs

calcine-geniex/src/
├─ cli/        discovery.rs (find geniex.exe) · runner.rs (spawn with fixed flags)
├─ backend/    models.rs · pull.rs · runtime.rs · serve.rs  (trait implementations)
└─ parse/      one parser per command output, tested on tests/fixtures/

calcine-gateway/src/
├─ security/   mod.rs (Host, Origin, auth middleware) · sanitize.rs (file paths, URLs)
├─ routes/     openai.rs (/v1) · manage.rs (/calcine/v1)
├─ proxy.rs    forwarding and streaming to geniex serve
├─ keys.rs     API keys (hashed, scoped) · caller.rs (who is calling)
└─ log.rs · settings.rs · state.rs · error.rs

calcine-hub/src/    hf.rs (Hugging Face payloads) · quant.rs (precision names) · aihub.rs (chipsets)

calcine-hw/src/     snapshot.rs · usage.rs (live load) · system.rs (sysinfo)
                    windows/ devices.rs (WMI) · gpu_engines.rs (PDH, the only `unsafe` code)
calcine-mock/src/   data.rs (sample data) · models.rs · directory.rs · runtime.rs · hardware.rs · server.rs

src-tauri/
├─ src/ipc/      mod.rs (bindings builder) · commands/ (one file per domain) · events.rs · error.rs
├─ src/setup/    services.rs (pick the backend) · gateway.rs · start and shutdown
├─ src/desktop/  tray.rs · window behaviour · notifications.rs · locale.rs · process.rs (Job Object)
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

## Frontend

```
src/
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
