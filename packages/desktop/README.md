# markers Desktop

Tauri desktop app for the markers GROBID pipeline: drop PDFs onto the window
(or pick them natively), and each one is converted to TEI XML + Markdown via a
local/remote GROBID service. Everything stays on your machine — no uploads.

## Layout

- `src/app/` — Next.js 16 static export (the whole UI is `/`; no landing
  page). Detects the Tauri runtime via `window.__TAURI__`; in a plain browser
  it shows a "desktop only" notice. Commands invoked: `pick_pdfs`,
  `pick_output_dir`, `convert_pdfs`, `convert_uploads`.
- `src-tauri/` — standalone Tauri 2 crate (own `[workspace]`, not part of the
  root Rust workspace). Delegates all conversion to `markers-grobid`
  (`pdf_to_tei`, `extract_metadata_from_tei`, `convert_tei_to_md`) and manages
  the GROBID backend (Docker / local JVM / `GROBID_URL`) through
  `grobid_client`.

## Commands

```bash
pnpm install
pnpm run dev      # Next.js dev server on :3000
pnpm run build    # static export → out/
pnpm run desktop  # tauri dev (needs Rust + a GROBID backend)
pnpm run tauri build   # production bundles
```

A GROBID backend is required only when TEI is missing: a running instance is
reused, `GROBID_URL` points at a remote one, `GROBID_LOCAL_BIN` (or the default
`~/.local/share/markers/grobid` install) spawns a local JVM service, and
otherwise a Docker container (`grobid/grobid` image) starts automatically.
See `scripts/setup-grobid-local.sh` and `crates/markers-grobid` for details.

## Conventions

- Keep `output: "export"` — Tauri serves `out/`; no server features.
- No `basePath`/`assetPrefix` — the app is not deployed to GitHub Pages.
- Tailwind CSS v4 + DaisyUI (`markers` theme) only; Inter via `next/font`.
- Rust deps exact-pinned (`=`); the src-tauri crate is intentionally outside
  the root workspace so `cargo build --workspace` stays CLI-only.
