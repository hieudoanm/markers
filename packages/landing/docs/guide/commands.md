---
title: Commands
order: 2
---

# Commands

`markers` has seven top-level commands. Run `markers <command> --help` for the
authoritative flag list.

## `markers build [content]`

Build a site. `[content]` defaults to `docs/`, output defaults to `dist/`.

```bash
markers build ./docs
markers build ./docs --output ./public
```

Every `.md`/`.markdown` file becomes a trailing-slash HTML page; every other
file is copied through as an asset. The build also writes `graph.html`
(page-link graph), `graph.json` and `graph.dot` data at the site root.

## `markers init [dir]`

Scaffold a new site: creates `dir/docs/index.md` and an example page. Defaults
to `./markers-site` when `dir` is omitted.

```bash
markers init my-site
```

## `markers serve [content]`

Build once, then serve the output on localhost — handy for checking clean URLs,
`basePath` handling and the theme toggle.

```bash
markers serve ./docs --port 8080
```

The server understands `basePath` (it strips the prefix before resolving files)
and redirects `/guide` → `/guide/`.

## `markers graph [content]`

Scan a content directory and write the link graph **without** building the site.
Emits three files next to each other:

- `graph.html` — the interactive viewer (same one embedded in builds)
- `graph.json` — nodes/edges as data
- `graph.dot` — Graphviz DOT

```bash
markers graph ./docs
```

## `markers wiki convert <url>`

Fetch a Wikipedia article through the MediaWiki REST API and print clean
Markdown. See [Wikipedia](../content/wiki.md) for the full pipeline.

```bash
markers wiki convert "https://en.wikipedia.org/wiki/Rust_(programming_language)"
markers wiki convert "<url>" -o rust.md
markers wiki convert "<url>" --format json
markers wiki convert "<url>" --include-references
markers wiki convert "<url>" --quiet        # scripting: no progress output
```

Flags: `-o/--output`, `--format markdown|json`, `--include-references`,
`-q/--quiet`.

## `markers wiki info <url>`

Show article metadata (title, canonical URL, language, revision id and
timestamp) without converting. `--format json` for machine-readable output.

```bash
markers wiki info "https://en.wikipedia.org/wiki/Rust_(programming_language)"
```

## `markers landing`

Generate a single-page static site from one `landing.yaml` — no build step, no
JavaScript, theme derived from eight colors. Subcommands:

| Command                    | Does                                                     |
| -------------------------- | -------------------------------------------------------- |
| `markers landing new`      | Scaffold an annotated `landing.yaml` (`-t` picks a type) |
| `markers landing validate` | Strictly validate the YAML schema                        |
| `markers landing build`    | Render the site files described below                    |
| `markers landing themes`   | List the 64 built-in theme presets                       |
| `markers landing preview`  | Serve the output directory over HTTP                     |

```bash
markers landing new --page-type product
markers landing validate -f landing.yaml
markers landing build -f landing.yaml -o public/index.html --theme ocean
markers landing preview --dir public --port 8080
```

Twelve page types: `product` (default), `waitlist`, `event`, `download`, `app`,
`pricing`, `portfolio`, `docs`, `faq`, `team`, `status`, `linktree`. `--theme`
overrides the YAML `theme:` section with a built-in preset; see
`markers landing themes` for all 64 names.

Page types, theming, the `landing.yaml` schema, and the full build output
(`index.html`, `styles.css`, `og/og.svg`, and the site metadata files) are
covered in [Landing pages](landing.md).

## `markers grobid`

Convert PDF articles to Markdown via
[GROBID](https://github.com/kermitt2/grobid) TEI XML, then index them into a
searchable corpus. Running `markers grobid` with no subcommand converts every
`*.pdf` in `./pdf` (resumable — existing TEI/Markdown outputs are reused).

| Command                                 | Does                                                          |
| --------------------------------------- | ------------------------------------------------------------- |
| `markers grobid [--enrich]`             | Convert `./pdf` → `./tei`, `./md`, and the `.grobid/` corpus  |
| `markers grobid convert`                | Same as above, explicit                                       |
| `markers grobid search <query>`         | Substring search over title/authors/journal/DOI/abstract/type |
| `markers grobid stats`                  | Works, references, year span, top authors/cited rankings      |
| `markers grobid graph [--db] [--out]`   | Write the citation dashboard to `.grobid/index.html`          |
| `markers grobid mcp [--http] [--token]` | Serve the corpus to AI agents over MCP (stdio or HTTP)        |

Converted Markdown gets YAML frontmatter (title, authors, year, journal,
volume/issue/pages, DOI) and a `## References` section with DOI-linked
citations. `--enrich` (or `GROBID_ENRICH=1`) adds Crossref `type`/`cited_by`.

A GROBID backend is required only when TEI files are missing: a running instance
is reused, `GROBID_URL` points at a remote one, `GROBID_LOCAL_BIN` (or the
default install path) spawns a local JVM service, and otherwise a Docker
container (`grobid/grobid` image) is started automatically.
`GROBID_DATA_DIR`/`ARTICLES_DATA_DIR` select the data directory; all corpus
artifacts land in `<data-dir>/.grobid/`.

## Exit codes

All commands exit non-zero on failure with a `markers <area>:`-prefixed error on
stderr — safe to use in scripts and CI.
