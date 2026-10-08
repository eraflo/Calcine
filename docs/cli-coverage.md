# GenieX CLI coverage

Which GenieX commands Calcine supports, and where. Update this table when a
command or flag becomes available in the app. Reference: GenieX v0.8.0.

| Command | Parser | Backend | UI | Notes |
|---|---|---|---|---|
| `list --format json` | ✅ `parse::list_json` | ✅ `ModelStore::list` | ✅ Library | Stable JSON schema |
| `version` | ✅ `parse::version` | ✅ `RuntimeManager::info` | ✅ Hardware, sidebar | |
| `config get chipset` | — (trimmed stdout) | ✅ `RuntimeManager::chipset` | ✅ Hardware, Welcome | |
| `config list` | ✅ `parse::config_list` | — | — | |
| `config set chipset` | — | — | — | M3: needs the chipset names list, only shown by the interactive picker today |
| `model list [--all]` | ✅ `parse::hub_table` | ✅ `ModelCatalog::aihub` | ✅ Discover, Welcome | `--all` lists chipset slugs (`x-elite`), not names |
| `pull <name>[:prec] [--model-hub] [--model-type]` | ✅ `parse::progress` (real capture) | ✅ `ModelStore::pull`, cancellable job | ✅ Discover, Welcome, task drawer | Without a TTY GenieX picks the recommended precision. `--local-path` in M3 |
| `remove --yes <keys…>` | — | ✅ `ModelStore::remove` | ✅ Library (model or one precision) | |
| `clean` | — | — | — | M3 |
| `model set-type` | — | — | — | M3 |
| `serve` | — | — | — | M2, behind the Calcine gateway |
| `infer` / `run` | — | — | — | Replaced by the Chat page over the HTTP API (M2) |
| `update` | — | — | — | M4, orchestrated by Calcine |
| `completion` | — | — | — | M5 |

Global flags Calcine always passes: `--skip-update`, plus `--data-dir` when
configured, with `NO_COLOR=1` in the environment.
