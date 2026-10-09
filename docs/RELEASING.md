# Releasing Calcine

Two channels, both installed and updated in place by Calcine itself
(Settings › Calcine updates):

| Channel | Comes from | Version | Published by |
|---|---|---|---|
| **Beta** | `dev`, on demand | `X.(Y+1).0-beta.<run>` | `.github/workflows/prerelease.yml` (run manually) |
| **Stable** | `main`, after a release PR | `X.Y.Z` | `.github/workflows/release.yml` |

Updates are signed with the updater key (minisign): the public key is in
`src-tauri/tauri.conf.json`, the private key and its password are the
repository secrets `TAURI_SIGNING_PRIVATE_KEY` and
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. **Losing the private key means
installed copies can't update anymore**: keep the backup in a password
manager.

## Stable release

1. On `dev`, with CI green:

   ```bash
   bun scripts/release.ts 1.0.0
   ```

   This sets the version in `Cargo.toml` and `package.json`, refreshes
   `Cargo.lock`, adds the `## 1.0.0` section to `CHANGELOG.md` from the
   Conventional Commits since the last release, and commits
   `chore(release): v1.0.0`. Edit the changelog wording if needed (amend).

2. Push `dev`, open a pull request `dev → main`, and run the manual checklist
   below on a Snapdragon PC (with a beta from that commit, run on demand, or a
   local `bun run build:release`).

3. Merge with a **merge commit** (not squash), so `dev` and `main` keep the
   same history.

4. `release.yml` sees a version on `main` without a release and a changelog
   section for it, then:
   - builds the NSIS installer on `windows-11-arm` (Calcine only: GenieX is
     downloaded on first launch, see below), and the Linux `.deb` on
     `ubuntu-24.04-arm` (`build-linux.yml`, which both release workflows call),
   - signs it with Authenticode through SignPath (once configured, below),
   - signs the updates and writes `latest.json` for the Stable channel
     (Windows and Linux),
   - attaches `SHA256SUMS`, a CycloneDX SBOM and a build provenance
     attestation (`gh attestation verify <file> -R eraflo/Calcine`),
   - publishes `vX.Y.Z` as the latest release, and offers it on the Beta
     channel too when it's newer than the latest beta.

Pushes to `main` that don't change the version publish nothing.

## Beta builds

Pushes to `dev` only run the checks (no installer). To publish a beta, run
**Actions › Prerelease › Run workflow** on `dev`. Each beta is a GitHub
pre-release with the installer, the `.deb` and `SHA256SUMS`; the fixed pre-release
`beta-channel` holds `latest-beta.json`, which the Beta channel reads.

The Windows and Linux ARM64 builds run in CI only for `main` and pull
requests to it. The Rust checks run on x86_64 Linux for every push.

## GenieX installed on first launch

Installers contain Calcine only (on Linux, GenieX's official archive is
unpacked to `~/.local/share/geniex-cli`, checked against its own pin). GenieX includes Qualcomm's proprietary
runtimes (QAIRT), which can't be signed or redistributed under SignPath
Foundation's terms, so the welcome screen downloads the official installer
from Qualcomm and checks it against the SHA-256 pinned in
`runtime/geniex.json`, built into Calcine.

`geniex-bump.yml` checks Qualcomm's release index daily. For a new stable
GenieX it verifies the installer (size and SHA-256 from the manifest),
installs it on a Windows ARM64 runner, checks `geniex version`, and opens a
PR to `dev` updating `runtime/geniex.json`. CI checks that the pinned
installer is still published (`scripts/check-geniex.ts`). PRs opened by workflows don't
start CI: close and reopen the PR to run the checks.

Users can also update GenieX from the app (Hardware › GenieX runtime), on the
stable or pre-release channel, independently of Calcine releases.

## Code signing (SignPath Foundation)

Installers aren't Authenticode-signed until the project is accepted by the
[SignPath Foundation](https://signpath.org) (free for open source). Until
then Windows SmartScreen warns before installing. Once accepted:

1. Create a project with an artifact configuration for the NSIS installer
   and a signing policy named `release-signing`.
2. Repository variables: `SIGNPATH_ORGANIZATION_ID`, `SIGNPATH_PROJECT_SLUG`.
3. Repository secret: `SIGNPATH_API_TOKEN`.

`release.yml` then signs every stable installer before computing the update
signature and checksums.

## Manual checklist (Snapdragon PC)

CI runners have no NPU. Before merging a release PR, with the matching beta:

- [ ] Fresh install on a PC without GenieX: GenieX gets installed, Welcome works.
- [ ] Update from the previous stable: settings, API keys and chats are kept.
- [ ] Download a model from AI Hub and one from Hugging Face; cancel and resume.
- [ ] Chat on the NPU (QAIRT) and on the GPU (llama.cpp); stop a reply.
- [ ] API: `curl` with a key works, without a key gets 401.
- [ ] Hardware: live load moves during a reply; self-test completes.
- [ ] GenieX update and roll back from Hardware › GenieX runtime.
- [ ] Calcine update from the previous version through Settings.
- [ ] Quit from the tray: no `geniex.exe` left running.
