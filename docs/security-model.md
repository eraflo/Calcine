# Security model

Calcine exposes your local models to other apps through an HTTP API on
`127.0.0.1:18181` (GenieX's default port, so existing GenieX clients work;
it can be changed in Settings › Local API).
This page explains what that API trusts and what it defends against.

## Architecture

```
other apps ──► Calcine gateway (127.0.0.1:18181) ──► geniex serve (127.0.0.1:<random port>)
                 Host · Origin · API key · body checks      no auth, internal only
```

`geniex serve` itself has no authentication and allows any browser origin
(`Access-Control-Allow-Origin: *`). Calcine therefore never exposes it: it runs
on a random loopback port, started on demand, with CORS restricted, and every
outside request goes through the gateway.

## What the gateway checks, in order

| Check | Defends against |
|---|---|
| Listens on `127.0.0.1` only | Other machines on the network |
| `Host` must be `127.0.0.1:<port>`, `localhost:<port>` or `[::1]:<port>` | DNS rebinding: a web page resolving its own domain to 127.0.0.1 to read the API |
| A browser `Origin`, when present, must be allowed (Calcine's webview, plus the origins listed in Settings › Local API) | Web pages calling the API from your browser (CSRF), even with simple requests that skip CORS preflight |
| `Authorization: Bearer calcine_…` must be a known key (on by default) | Other local programs using your models without your consent |
| Key scopes: `inference` (`/v1/*`) and `manage` (`/calcine/v1/*`) | An app meant to chat downloading or deleting models |
| Request bodies: local paths and URLs refused in `image_url`, `grammar_path`, `spec_draft_model` unless the key allows local files (`input_audio` is always decoded as base64 by GenieX, so it needs no check) | GenieX reading arbitrary files (`C:/Users/…`) or fetching internal URLs (SSRF) on an app's behalf |
| Model names used to count tokens: only `owner/name[:precision]` made of letters, digits, `-`, `_` and `.`, never `..`, read inside GenieX's model cache | A client making Calcine read `tokenizer.json` or `genie_config.json` outside the cache |
| 64 MiB body limit | Memory exhaustion |

Errors use the OpenAI shape (`{"error": {"message", "type"}}`) so client
libraries show the reason.

## API keys

- One key per app, created in **Server → API keys**, with a name and scopes.
- Shown once at creation; Calcine stores only a SHA-256 fingerprint in
  `api-keys.json` in its data folder, compared in constant time.
- Revoking a key takes effect immediately.
- Calcine's own UI uses a separate token generated at each launch, kept in
  memory and given only to its webview over IPC.
- **Require an API key** can be turned off for clients that can't send one. In
  that mode, keyless callers can only run models: no management, no local files.

## Ollama apps

Ollama's API (`/api/chat`, `/api/generate`, `/api/tags`, ...) is translated to
GenieX's OpenAI API. On the main port it follows the rules above. Settings ›
Local API › **Ollama apps** also opens Ollama's port, `127.0.0.1:11434`, for
apps that can't send a key:

- The same Host and Origin checks apply: other machines and web pages are
  refused.
- Callers without a key there can only run models (no management, no local
  files), like keyless callers when keys aren't required. They show up as
  "Ollama app" in the request log.
- Which port a request came in on is decided by the socket it arrived on,
  never by its headers, so a request to the main port can't pass for an
  Ollama one.
- It's off by default. If Ollama itself is running, the port is taken and
  Calcine says so.

## Privacy

The request log records who called which model, the status, duration, token
counts and speed. Prompts and replies are never recorded or sent anywhere.
Chat conversations are stored in the app's local storage on this PC only.

## Process lifetime

Calcine runs inside a Windows Job Object that kills its child processes
(`geniex serve`, downloads) when Calcine exits, including after a crash, so no
unauthenticated server or loaded model is left behind.

## Updates

- **GenieX**: Calcine reads Qualcomm's release index over HTTPS, downloads the
  installer, and refuses it unless its SHA-256 matches the official manifest.
  An Authenticode signature, when present, must be valid and from Qualcomm
  (GenieX installers aren't signed yet; `windows-signed.txt` is reported in
  the UI). `geniex serve` is stopped before the installer runs.
- **Calcine**: updates are signed with Calcine's updater key (minisign) and
  verified by the app before installing; the public key ships in the app.
  Release installers also carry SHA-256 checksums, a CycloneDX SBOM and a
  GitHub build provenance attestation.
- Links Calcine opens in the browser are limited to HTTPS pages on GitHub,
  Hugging Face and Qualcomm AI Hub.

## Not in scope (yet)

- Exposing the API on the local network (planned: opt-in, TLS and keys required).
- Rate limiting per key.
- Programs running as your Windows user can read Calcine's data folder or talk
  to `geniex serve` directly if they find its port; they already have your
  privileges, so this is outside what a local API can defend against.

Found a problem? See [SECURITY.md](../SECURITY.md).
