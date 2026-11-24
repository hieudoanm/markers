---
title: Install the CLI on macOS (Intel)
description: Download and install the markers binary for macOS (Intel).
order: 60
---

# How to install the markers CLI on macOS (Intel)

For Intel-based Macs.

## Option 1: manual download (recommended)

Prebuilt binaries are published to the
[latest release](https://github.com/hieudoanm/markers/releases/latest).

1. Open the [downloads page](https://hieudoanm.github.io/markers/downloads/) and
   download **cli-markers-x86_64-apple-darwin** under **CLI** —
   `cli-markers-x86_64-apple-darwin`.
2. Make it executable and move it onto your `PATH`:

```sh
chmod +x cli-markers-x86_64-apple-darwin
mv cli-markers-x86_64-apple-darwin ~/.local/bin/markers
```

3. Verify:

```sh
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

- `markers: command not found` — the install directory is not on your `PATH`.
  Add `export PATH="$HOME/.local/bin:$PATH"` to your shell profile.
- Gatekeeper warning — the binary is unsigned. Right-click → Open, or run
  `xattr -d com.apple.quarantine` on the installed binary.
