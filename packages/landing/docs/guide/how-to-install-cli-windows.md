---
title: Install the CLI on Windows
description: Download and install the markers binary for Windows.
order: 60
---

# How to install the markers CLI on Windows

For 64-bit Windows.

## Option 1: manual download (recommended)

Prebuilt binaries are published to the
[latest release](https://github.com/hieudoanm/markers/releases/latest).

1. Open the [downloads page](https://hieudoanm.github.io/markers/downloads/) and
   download **Windows** under **CLI** —
   `cli-markers-x86_64-pc-windows-msvc.exe`.
2. Rename it to `markers.exe` and place it in a directory on your `PATH` (for
   example `%LOCALAPPDATA%\Programs\markers\` or `C:\tools\`).
3. Open a new terminal and verify:

```powershell
markers --version
```

## Option 2: install script (builds from source)

`scripts/install.sh` does not download a prebuilt binary — it fetches the source
tarball and compiles with Cargo, so it needs a
[Rust toolchain](https://rustup.rs/):

```sh
curl -fsSL https://raw.githubusercontent.com/hieudoanm/markers/master/scripts/install.sh | sh
```

The script installs `markers` and `markers-mcp` under `~/.local` (override with
`PREFIX=/path`).

## Troubleshooting

- `markers is not recognized` — the directory is not on `PATH`. Add it via
  Settings → Environment Variables, or call the binary by full path.
- SmartScreen warning — the binary is unsigned. Choose **More info → Run
  anyway**.
