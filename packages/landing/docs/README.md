---
title: Documentation
order: 0
---

# markers documentation

`markers` turns a folder of Markdown files into a complete static website —
clean URLs, generated navigation, light/dark themes, and an interactive graph of
how pages link together. No build pipeline, no runtime, no configuration file
required: **one Rust binary**.

```bash
markers build ./docs
```

## Quick start

```bash
markers init my-site          # scaffold docs/index.md + an example page
markers serve my-site/docs    # build + serve on http://localhost:8080
markers build my-site/docs --output my-site/dist
```

See [Getting started](guide/getting-started.md) for installation and your first
site.

## Features

- **[Zero configuration](guide/features.md#zero-configuration)** — the directory
  layout is the config; frontmatter covers ordering, labels and drafts.
- **[Clean URLs](guide/features.md#clean-urls-zero-rewrite-rules)** — every page
  publishes at `/section/page/`, so `dist/` works on any static host.
- **[File-tree navigation](guide/features.md#file-tree-navigation)** — sidebar
  generated from folders and frontmatter.
- **[GitHub-flavoured Markdown](guide/features.md#github-flavoured-markdown-extended)**
  — tables, alerts, footnotes, heading ids and rewritten internal links.
- **[Eight themes](guide/features.md#themes-out-of-the-box)** — palettes
  embedded in the binary, plus a persisted light/dark toggle.
- **[Graph page](guide/features.md#the-graph-page)** — interactive map of
  page-to-page links on every build.
- **[Wikipedia import](guide/features.md#wikipedia-markdown)** — article URL →
  clean Markdown, with a BFS crawler for whole sections.
- **[MCP server](guide/features.md#mcp-server)** — `markers-mcp` exposes build
  and inspection tools to AI agents over stdio.

## Guides

- **[Getting started](guide/getting-started.md)** — install, scaffold, build.
- **[Commands](guide/commands.md)** — every CLI command and flag.
- **[Configuration](guide/configuration.md)** — frontmatter, `markers.yaml`,
  `basePath`, URL rules.
- **[Markdown](content/markdown.md)** — GFM syntax, heading ids, alerts, link
  rewriting.
- **[Theming](content/theming.md)** — palettes, tokens, the dark-mode toggle.
- **[Graph](content/graph.md)** — the page-link graph and `markers graph`
  outputs.
- **[Wikipedia](content/wiki.md)** — `markers wiki convert` /
  `markers wiki info`.
- **[MCP server](project/mcp.md)** — client setup and the five `markers_*`
  tools.
- **[Scripts](project/scripts.md)** — installers and the `examples.sh` crawler.
- **[Deploy](project/deploy.md)** — GitHub Pages workflow and other static
  hosts.

## Internals

- **[Architecture](project/architecture.md)** — the Cargo workspace, the build
  pipeline, and how to develop `markers` itself.

## Installation guides

- [CLI on macOS (Apple Silicon)](guide/how-to-install-cli-macos-silicon.md)
- [CLI on macOS (Intel)](guide/how-to-install-cli-macos-intel.md)
- [CLI on Linux](guide/how-to-install-cli-linux.md)
- [CLI on Windows](guide/how-to-install-cli-windows.md)
