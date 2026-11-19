# markers-cli — Agent Guide

Back to the [workspace README](../../README.md) · [docs](../../packages/landing/docs/).

## Conventions

- This crate is a **binary** — keep it thin. Business logic belongs in the
  library crates (`markers-build`, `markers-wiki`, `markers-site`, …); the CLI
  only parses args, calls the library, formats output, sets exit codes.
- clap derive for all arguments; match the existing command/flag naming
  (`--output`, `--format`, `--quiet`).
- Errors print as `markers <area>: <message>` on stderr and exit non-zero.
- `serve.rs` must stay `basePath`-aware: strip the configured prefix before
  resolving files, and redirect `/dir` → `/dir/` for clean URLs.

## Verification

```bash
cargo build -p markers-cli        # produces target/{debug,release}/markers
cargo clippy -p markers-cli
cargo run -p markers-cli -- --help
```
