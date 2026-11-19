# markers-cli

The `markers` command-line binary — clap-based entry point that wires the
library crates together: `build`, `init`, `serve`, `graph`, and the
`wiki convert`/`wiki info` commands.

Part of the [markers](../../README.md) workspace; see the
[documentation](../../packages/landing/docs/) for the user-facing guide.

## Commands

| Module     | Command                                                                       |
| ---------- | ----------------------------------------------------------------------------- |
| `main.rs`  | clap definitions + dispatch                                                   |
| `build.rs` | thin wrapper over `markers-build::build`                                      |
| `init.rs`  | `markers init` site scaffolding                                               |
| `serve.rs` | `markers serve` static file server (basePath-aware, trailing-slash redirects) |
| `wiki.rs`  | `markers wiki convert`/`info` CLI surface over `markers-wiki`                 |

## Usage

```bash
cargo run -p markers-cli -- build ./docs --output ./dist
cargo run -p markers-cli -- wiki convert "https://en.wikipedia.org/wiki/Rust_(programming_language)"
```

The package ships `[[bin]] name = "markers"` — the binary is `markers`, not
`markers-cli`.
