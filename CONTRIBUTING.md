# Contributing to markers

Thanks for contributing to markers, a static site generator for documentation —
point it at a folder of Markdown and get a complete static site plus a link
graph.

## Repository layout

A Cargo workspace with two binaries and six library crates under `crates/`:

- `markers-cli` — the `markers` binary (build, serve, init, wiki).
- `markers-mcp` — the `markers-mcp` MCP server over stdio.
- `markers-build`, `markers-site`, `markers-markdown`, `markers-theme`,
  `markers-graph`, `markers-wiki` — pipeline, discovery, rendering, graph, and
  Wikipedia import.
- `packages/landing` — docs site deployed to GitHub Pages.

Dependency direction: `markers-cli`/`markers-mcp` → `markers-build`/
`markers-theme`/`markers-graph` → `markers-markdown` → `markers-site`. Keep it
acyclic.

## Setup

```bash
git clone https://github.com/hieudoanm/markers.git
cd markers
cargo build
```

## Checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Conventions

- The directory layout is the configuration — no config file beyond optional
  `markers.yaml` and frontmatter.
- Internal Markdown links must be rewritten to clean trailing-slash URLs;
  broken links stay untouched so they remain visible.
- Keep crates lean; errors are `Result<_, String>`.
- Update `README.md` and `packages/landing/docs/` when behaviour changes.

## License

By contributing, you agree that your contributions will be licensed under the
repository license in [`LICENSE`](LICENSE).
