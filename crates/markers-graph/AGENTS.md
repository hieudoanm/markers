# markers-graph — Agent Guide

Back to the [workspace README](../../README.md) · [Graph docs](../../packages/landing/docs/graph.md).

## Conventions

- Two entry points, one model: `from_pages` (edges from the build) and
  `scan` (standalone directory scan for `markers graph`) must produce the
  same `Graph` shape — reuse `markers_site`'s `*_pub` helpers for path/title
  derivation instead of re-implementing.
- The viewer is **dependency-free vanilla JS/canvas** — deliberately not
  vis-network or similar; keep `graph.js` self-contained and small.
- `render_html` output must stay a single file (inline or embedded assets)
  so `graph.html` works when opened directly.
- Node `id` is the page URL (`/guide/install/`); edges reference those ids —
  keep them consistent with `markers_site::site::url_for_source`.

## Verification

```bash
cargo test -p markers-graph
cargo run -p markers-cli -- graph docs   # inspect graph.html/.json/.dot
```
