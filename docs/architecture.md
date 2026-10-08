# Architecture

```
 Other apps ──HTTP──┐                       ┌── React UI (WebView2)
 (OpenAI SDKs…)     │                       │     │ typed commands (tauri-specta)
                    ▼                       │     ▼
           ┌─────────────────┐     ┌──────────────────────┐
           │ calcine-gateway │     │ src-tauri (shell)    │
           │ :18181 (M2)     │     │ thin commands, tray  │
           └────────┬────────┘     └──────────┬───────────┘
                    └──────────┬──────────────┘
                               ▼
                   ┌────────────────────────┐
                   │ calcine-core::Services │  single source of truth
                   └───────────┬────────────┘
                               ▼ trait implementations
               ┌───────────────┴───────────────┐
               ▼                               ▼
        calcine-geniex                   calcine-mock
        spawns `geniex …`                fake data (any OS, CI)
```

## Crates

| Crate | Role | Depends on Tauri? |
|---|---|---|
| `crates/calcine-core` | Domain types, model references (pasted names and links), service traits (`ModelStore`, `ModelCatalog`, `RuntimeManager`, `HardwareProbe`), the `JobManager` for long-running work, errors, `Services` | No |
| `crates/calcine-geniex` | GenieX CLI adapter: discovery, command runner, `pull` with live progress, output parsers tested on real captures (`tests/fixtures/`) | No |
| `crates/calcine-hw` | Hardware probe: CPU, memory and disk (sysinfo), NPU/GPU and drivers (Windows WMI) | No |
| `crates/calcine-mock` | In-memory backend with simulated downloads, for UI work and tests | No |
| `src-tauri` | Desktop shell: builds `Services`, exposes commands, exports TypeScript bindings | Yes |

Backend selection: `CALCINE_BACKEND=mock` uses the mock, anything else uses GenieX.

## Jobs

Downloads (and later updates and benchmarks) run as jobs:
`Services::start_pull` spawns one through the `JobManager`, which tracks its
state and progress, supports cancellation, and broadcasts every update. The
Tauri shell forwards updates to the webview as the typed `JobUpdated` event
(throttled to ~5 per second per job); the UI mirrors them into the
TanStack Query cache (`src/features/tasks/api.ts`).

## Desktop behavior

- Closing the window hides it to the tray; quit from the tray menu.
- Launching Calcine again focuses the running instance (single-instance plugin).

## Frontend

```
src/
├─ app/            router, navigation, shell (sidebar, topbar, ⌘K palette, task drawer)
├─ features/<x>/   one folder per page: api.ts (TanStack Query) + components
├─ components/ui/  shadcn/ui-style primitives, themed with Ember tokens
├─ components/calcine/  app-specific building blocks (badges, page frame, states)
├─ stores/         Zustand stores (UI state, persisted theme)
├─ styles/         globals.css: Ember design tokens (dark default, light variant)
└─ lib/            bindings.ts (generated), api.ts (unwrap, errors), utils
```

## Adding a feature

1. Add the domain method to a trait in `calcine-core` and implement it in
   `calcine-geniex` and `calcine-mock` (with a parser test on real output).
2. Add a thin command in `src-tauri/src/commands/` and register it in
   `src-tauri/src/bindings.rs`.
3. Run `cargo test -p calcine` to regenerate `src/lib/bindings.ts` (CI fails if
   it's stale).
4. Build the UI in `src/features/<feature>/` and, for a new page, add its route
   in `src/app/router.tsx` and its entry in `src/app/navigation.ts`.

## Tests that link Tauri

On Windows, only integration test binaries get the app manifest Tauri needs
(see `src-tauri/build.rs`). Put tests that link Tauri in `src-tauri/tests/`,
not in `src-tauri/src/`.
