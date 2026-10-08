# Third-party notices

Calcine is licensed under the MIT License (see [LICENSE](LICENSE)).

`crates/calcine-hub/src/quant.rs` follows the precision naming rules of
GenieX's model manager (`sdk/model-manager/crates/core/src/manifest_builder.rs`,
BSD 3-Clause, license below) so that Calcine names precisions exactly like
`geniex pull` does.

## GenieX CLI

- **Project**: https://github.com/qualcomm/GenieX
- **Not redistributed**: Calcine's installer doesn't contain GenieX. On first
  launch, and when you update it, Calcine downloads Qualcomm's official
  `geniex-cli-setup-windows-arm64-<version>.exe` from Qualcomm's release bucket,
  checks it against the SHA-256 pinned in `runtime/geniex.json` (or the release
  manifest, for updates), and runs it. GenieX includes its own runtimes
  (Qualcomm AI Engine Direct / QAIRT and llama.cpp); see the
  [GenieX NOTICE](https://github.com/qualcomm/GenieX/blob/main/NOTICE) for their
  licenses.
- **geniex-bench**: downloaded on demand from GenieX's GitHub release, not
  redistributed either.
- **License**: BSD 3-Clause

```
BSD 3-Clause License

Copyright (c) 2024-2026, Qualcomm Technologies, Inc. and/or its subsidiaries.
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its
   contributors may be used to endorse or promote products derived from
   this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```

Calcine is an independent project and is not affiliated with or endorsed by
Qualcomm Technologies, Inc.

## Rust and JavaScript dependencies

Licenses of compiled-in Rust crates and npm packages are checked in CI
(`cargo-deny`) and listed in the SBOM attached to each GitHub release.
