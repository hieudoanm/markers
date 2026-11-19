# markers-grobid

PDF articles → Markdown via [GROBID](https://github.com/kermitt2/grobid) TEI
XML, plus a SQLite corpus index with search, stats, a citation-graph
dashboard, and an MCP server. Powers `markers grobid …`.

Ported from the standalone `grobid.rs` project.

## Pipeline

```text
pdf/ → GROBID processFulltextDocument → tei/*.tei.xml
     → Markdown with YAML frontmatter → md/*.md
     → <data-dir>/.grobid/ corpus (grobid.db, index.json,
       metadata/*.json, process.json, index.html dashboard)
```

Backend precedence: running instance → `GROBID_URL` (external, never
managed) → local JVM `grobid-service` (`GROBID_LOCAL_BIN` or the default
install path) → Docker container (`grobid/grobid` image).

## Modules

| Module                 | Role                                                                            |
| ---------------------- | ------------------------------------------------------------------------------- |
| `config.rs`            | `GROBID_*` env config: port, URL, image, container, local bin, data dir         |
| `paths.rs`             | `pdf/`/`tei/`/`md/` dirs + `.grobid/` artifact paths                            |
| `grobid_client/`       | Backend management (`mod`/`docker`/`local`) + `pdf_to_tei` conversion           |
| `metadata/`            | TEI `teiHeader` → `Metadata` + YAML frontmatter                                 |
| `render/`              | TEI → Markdown (`tei`/`inline`/`blocks`/`references`/`convert`)                 |
| `corpus/`              | SQLite `works`+`refs` index: upsert, LIKE search, stats, graph, JSON export     |
| `enrich.rs`            | Optional Crossref metadata enrichment (`--enrich`/`GROBID_ENRICH`)              |
| `cli.rs`               | Orchestration: convert pipeline, `graph`/`search`/`stats` commands              |
| `mcp.rs`               | MCP server (stdio or streamable HTTP): `grobid_*` tools + `grobid://` resources |
| `templates/graph.html` | Self-contained corpus dashboard (d3-force + stats + year chart)                 |

## CLI surface (via markers-cli)

```bash
markers grobid                 # convert all PDFs in ./pdf (resumable)
markers grobid convert         # same, explicit
markers grobid --enrich        # + Crossref enrichment
markers grobid search <query>  # corpus search
markers grobid stats           # corpus statistics
markers grobid graph           # write .grobid/index.html dashboard
markers grobid mcp             # MCP over stdio
markers grobid mcp --http 127.0.0.1:PORT [--token …]
```

## Tests

`tests/unit/` covers render/metadata/corpus/enrich against inline TEI plus
the real fixtures in `tests/fixtures/` (8 stroke/neuroimaging papers).
`tests/e2e.sh` runs the full pipeline against real GROBID (needs Docker or a
local `grobid-service`; set `E2E_ENRICH=1` for Crossref).
