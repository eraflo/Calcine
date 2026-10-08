# GenieX CLI coverage

Which GenieX commands Calcine supports, and where. Update this table when a
command or flag becomes available in the app. Reference: GenieX v0.8.0.

| Command | Parser | Backend | UI | Notes |
|---|---|---|---|---|
| `list --format json` | ✅ `parse::list_json` | ✅ `ModelStore::list` | ✅ Library | Stable JSON schema |
| `version` | ✅ `parse::version` | ✅ `RuntimeManager::info` | ✅ Hardware, sidebar | |
| `config list` / `get` | ✅ `parse::config_list` | — | — | Chipset (M1) |
| `config set chipset` | — | — | — | M1 |
| `model list [--all]` | ✅ `parse::hub_table` | — | — | AI Hub catalog (M1); `--all` uses chipset slugs |
| `pull` | — | — | — | M1, progress parser to build from a real capture |
| `remove` / `rm` | — | — | — | M1 |
| `clean` | — | — | — | M3 |
| `model set-type` | — | — | — | M3 |
| `serve` | — | — | — | M2, behind the Calcine gateway |
| `infer` / `run` | — | — | — | Replaced by the Chat page over the HTTP API (M2) |
| `update` | — | — | — | M4, orchestrated by Calcine |
| `completion` | — | — | — | M5 |

Global flags Calcine always passes: `--skip-update`, plus `--data-dir` when
configured, with `NO_COLOR=1` in the environment.
