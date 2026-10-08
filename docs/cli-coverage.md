# GenieX CLI coverage

Which GenieX commands Calcine supports, and where. Update this table when a
command or flag becomes available in the app. Reference: GenieX v0.8.0.

| Command | Parser | Backend | UI | Notes |
|---|---|---|---|---|
| `list --format json` | ✅ `parse::list_json` | ✅ `ModelStore::list` | ✅ Library | Stable JSON schema |
| `version` | ✅ `parse::version` | ✅ `RuntimeManager::info` | ✅ Hardware, sidebar | |
| `config get chipset` | — (trimmed stdout) | ✅ `RuntimeManager::chipset` | ✅ Hardware, Welcome | Prints the reference device when detected, the saved id when set |
| `config list` | ✅ `parse::config_list` | — | — | |
| `config set chipset <id>` / `""` | — | ✅ `RuntimeManager::set_chipset` | ✅ Hardware › Chipset | Always passes a value (an empty one resets to detection), so the interactive picker never opens. Chipset names come from AI Hub's `platform.json` (`calcine-hub`), like the picker |
| `model list [--all]` | ✅ `parse::hub_table` | ✅ `ModelCatalog::aihub` | ✅ Discover › Qualcomm AI Hub, Welcome | `--all` lists chipset slugs (`x-elite`), not names |
| `pull <name>[:prec] [--model-hub] [--model-type]` | ✅ `parse::progress` (real capture) | ✅ `ModelStore::pull`, cancellable job | ✅ Discover (precision picker with sizes, memory and disk checks, type override), Welcome, task drawer | Precisions and sizes come from the Hugging Face API (`calcine-hub`), named like GenieX does. Without a TTY GenieX picks the recommended precision |
| `pull <name> --model-hub localfs --local-path <dir\|zip>` | ✅ (same progress) | ✅ `PullRequest::import`, `Import` job | ✅ Library › Import, drag and drop | GGUF folders, extracted AI Hub bundles, AI Hub `.zip` |
| `remove --yes <keys…>` | — | ✅ `Services::remove_models` | ✅ Library (model or one precision) | Stops `geniex serve` first (Windows can't delete a loaded model) |
| `clean --yes` | — | ✅ `Services::clean_models` | ✅ Settings › Storage › Danger zone | Typed confirmation; stops `geniex serve` first |
| `model set-type <name> <llm\|vlm>` | — | ✅ `ModelStore::set_type` | ✅ Library (Text / Vision on each model) | Always passes the type, so the interactive picker never opens |
| `serve --host 127.0.0.1:<random> --origins --keepalive --nctx` | — | ✅ `InferenceServer` (`GeniexServer`, supervised), `ServerOptions` saved in `server.json` | ✅ Server page (start/stop, output, Model loading: unload delay and llama.cpp context window), on demand from the gateway | Never exposed directly; see security-model.md. Changing the options restarts it. `nctx` sent per request is ignored by GenieX 0.8, hence a server option |
| `infer` / `run` | — | via `/v1/chat/completions` | ✅ Chat (streaming, reasoning, presets, temperature, top-p/k, min-p, penalties, seed, stop sequences, max tokens, think, compute, offloaded layers, vision encoder, power mode, saved system prompts, images and microphone for VLMs), Hardware › Self-test | The REPL itself isn't wrapped; the HTTP API takes the same options. Measured on GenieX 0.8: `seed` and `ngl` only apply to llama.cpp models, `vit_compute` takes `CPU` or `HTP0` (not the GPU), and `stop` is ignored over the API, so the Chat cuts the reply itself. Speculative decoding (`spec_type` as one comma-separated string, `spec_draft_model` by name, `spec_n_max/min`, `spec_p_min`; acceptance from `timings.draft_n`/`draft_n_accepted`) in the Chat and the Benchmark, llama.cpp only. Not yet: grammars (M5) |
| `update` | — | ✅ `RuntimeInstaller` (`calcine-geniex::update`), `Services::start_runtime_install` | ✅ Hardware › GenieX runtime › Updates (stable / pre-releases, roll back, repair), sidebar badge | Not wrapped: Calcine reads the same `index.json` and manifests, downloads the installer with progress, checks its SHA-256 and Authenticode signature, stops `geniex serve`, runs the installer silently, checks `geniex version`, restarts the server. Installers are cached to roll back |
| `completion` | — | — | — | M5 |
| `geniex-bench` (separate tool) | `calcine-geniex::bench::report` (JSON report schema 6, `ERROR:` lines) | ✅ `Benchmarker` (`GeniexBench`), `Services::start_benchmark`, history in `benchmarks.json` | ✅ Benchmark page (download, run per unit, results, fastest models, history, CSV export) | Downloaded from GenieX's GitHub release at the installed GenieX version; GitHub's `digest` and the published `.sha256` must agree. Unpacked with Windows' `tar`. One process per unit (in matrix mode a failure stops the rest). `geniex serve` is paused meanwhile |

Global flags Calcine always passes: `--skip-update`, plus `--data-dir` when
configured, with `NO_COLOR=1` in the environment.

## Outside the CLI

| What | Where | Notes |
|---|---|---|
| Ollama API (`/api/version`, `tags`, `show`, `ps`, `chat`, `generate`) | `calcine-gateway` › `ollama.rs`, `routes/ollama.rs` | Translated to `/v1/chat/completions` (images, options, `think`, `format`, tools), answered as JSON lines or one object. On the main port with a key, and on 11434 without one when turned on (Settings › Local API). Model management (`pull`, `delete`, ...) answers 501: it stays in the app |
| Hugging Face search (GGUF) | `calcine-hub` › `ModelDirectory::search` | `GET /api/models?filter=gguf`, honours `HF_ENDPOINT` |
| Precisions with download sizes | `calcine-hub` › `ModelDirectory::details` | `GET /api/models/{repo}?blobs=true`; shards summed, vision projector added |
| AI Hub chipsets | `calcine-hub` › `ModelDirectory::chipsets` | `releases/latest/platform.json`, filtered to the host OS like GenieX |
| NPU / GPU / CPU load | `calcine-hw` › `HardwareProbe::usage` | `GPU Engine` performance counters (PDH): compute-only adapter = NPU |
