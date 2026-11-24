---
title: Install the CLI on Linux
description: Download and install the markers binary for Linux.
order: 60
---

# How to install the markers CLI on Linux

For x86_64 Linux distributions.

## Option 1: manual download (recommended)

Prebuilt binaries are published to the
[latest release](https://github.com/hieudoanm/markers/releases/latest).

1. Open the [downloads page](https://hieudoanm.github.io/markers/downloads/) and
   download **cli-markers-x86_64-unknown-linux-gnu** under **CLI** —
   `cli-markers-x86_64-unknown-linux-gnu`.
2. Make it executable and move it onto your `PATH`:

```sh
chmod +x cli-markers-x86_64-unknown-linux-gnu
mv cli-markers-x86_64-unknown-linux-gnu ~/.local/bin/markers
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

- `markers: command not found` — ensure `~/.local/bin` is on your `PATH` (most
  distributions already include it).
- `No such file or directory` on a glibc system — the binary is built against
  glibc; on musl-based distros (Alpine) build from source instead.
