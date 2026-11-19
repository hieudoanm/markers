# markers-grobid — Agent Guide

Back to the [workspace README](../../README.md).

## Conventions

- Ported from the `grobid.rs` project — keep module boundaries
  (`grobid_client`, `metadata`, `render`, `corpus`) intact; the TEI parser
  matches elements by **local name** (namespace-agnostic) via
  `descendant_named`/`descendants_named`/`tei_root` — do not reintroduce
  xpath-string matching.
- Metadata author/DOI/journal lookups are scoped to `teiHeader` on purpose —
  document-wide search picks up reference-list authors.
- Resumable pipeline: existing `tei/*.tei.xml` and `md/*.md` are skipped;
  the convert command exits non-zero if any file failed.
- HTTP goes through `reqwest` clients built in `cli.rs`/`grobid_client` —
  health probes use a short-timeout client; the convert client has a 300s
  timeout; Crossref uses a 15s client with a `markers/` user agent.
- `GROBID_*` env vars configure the external GROBID service — keep the names;
  they describe the upstream tool, not this crate.
- Corpus artifacts live in `<data-dir>/.grobid/` (db, index.json,
  metadata/, process.json, index.html). `markers grobid` is exposed through
  `markers-cli`'s thin clap layer — business logic stays here.
- MCP tools keep the `grobid_*` prefix and `grobid://` resource URIs.

## Verification

```bash
cargo test -p markers-grobid
cargo clippy -p markers-grobid --all-targets -- -D warnings
cargo run -p markers-cli -- grobid stats   # needs a corpus DB (GROBID_DATA_DIR)
```
