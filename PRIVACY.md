# Privacy policy

Calcine runs language models on your PC. Your prompts, replies, images,
recordings and conversations never leave it. Calcine has no account, no
telemetry, no analytics and no crash reporting.

Calcine does connect to the internet for the things listed below. Each
connection is a normal HTTPS request: the service sees your IP address, the
address requested and a user agent, and nothing else from Calcine.

## Connections Calcine makes

| When | Where | What for |
|---|---|---|
| At startup, if **Check for updates automatically** is on (the default) | `github.com` (Calcine's releases) | See whether a newer Calcine is available |
| At startup, if **Check for updates automatically** is on (the default) | `qaihub-public-assets.s3.us-west-2.amazonaws.com` (Qualcomm) | See whether a newer GenieX is available |
| When you install, update or repair GenieX (also offered on first launch) | `qaihub-public-assets.s3.us-west-2.amazonaws.com` (Qualcomm) | Download the official GenieX installer, checked against its published SHA-256 |
| When you install a Calcine update | `github.com` | Download the update, checked against Calcine's signature |
| When you browse **Discover** or change the chipset on **Hardware** | `huggingface.co`, `qaihub-public-assets.s3.us-west-2.amazonaws.com` (Qualcomm AI Hub) | Search models, read their precisions and sizes, list the chipsets they're built for |
| When you download a model | Hugging Face, ModelScope, Docker Hub or Qualcomm AI Hub, depending on the model | GenieX downloads the model you asked for |
| When you open **Benchmark** or download its tool | `api.github.com`, `github.com` (GenieX's releases) | Find and download Qualcomm's `geniex-bench`, checked against its published SHA-256 |
| When you click a link | Your web browser opens GitHub, Hugging Face or Qualcomm AI Hub | Release notes and model pages |

Turn off **Settings › Calcine updates › Check for updates automatically** and
Calcine connects only when you ask it to: updates are then checked when you
click **Check now**.

Calcine starts GenieX with `--skip-update`, so GenieX doesn't check for updates
by itself.

These services have their own privacy policies:
[GitHub](https://docs.github.com/site-policy/privacy-policies/github-general-privacy-statement),
[Hugging Face](https://huggingface.co/privacy),
[Qualcomm](https://www.qualcomm.com/site/privacy) (its files are served by
[Amazon Web Services](https://aws.amazon.com/privacy/)).

## Data kept on your PC

- **Conversations and chat settings**: in Calcine's web storage, on this PC.
- **API keys**: only a SHA-256 fingerprint of each key, in `api-keys.json`.
- **Request log**: who called which model, when, how fast and how many tokens,
  never the prompts or replies. Kept in memory and lost when Calcine closes.
- **Settings and benchmark results**: JSON files in Calcine's data folder.
- **Models**: in GenieX's model folder.

Calcine's data folder is `%APPDATA%\com.eraflo.calcine` (settings, keys,
benchmarks) and `%LOCALAPPDATA%\com.eraflo.calcine` (web storage, downloaded
installers and tools). Uninstalling Calcine can remove them.

## The local API

Calcine's API listens on `127.0.0.1` only, so other machines can't reach it.
Apps on this PC need an API key unless you turn that off, and web pages are
refused unless you allow their origin. See the
[security model](docs/security-model.md).

## Contact

Questions about this policy: open an issue at
https://github.com/eraflo/Calcine/issues.
