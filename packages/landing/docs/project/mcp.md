---
title: MCP server
order: 8
---

# MCP server

`markers-mcp` is a stdio
[Model Context Protocol](https://modelcontextprotocol.io) server that exposes
markers' operations as tools — build sites, list pages, render Markdown, inspect
the link graph — so MCP-capable agents (and other clients) can drive the
generator directly.

## Setup

Point your MCP client at the binary:

```json
{
  "mcpServers": {
    "markers": {
      "command": "markers-mcp"
    }
  }
}
```

Install it with the same scripts as `markers` (`install-local.sh`, `install.sh`,
or `cargo install --path crates/markers-mcp`).

## Tools

All tools return pretty-printed JSON; failures surface as MCP errors.

| Tool                      | Parameters                    | Returns                                                 |
| ------------------------- | ----------------------------- | ------------------------------------------------------- |
| `markers_build`           | `content_dir?`, `output_dir?` | Build summary (pages, assets, graph)                    |
| `markers_list_pages`      | `content_dir?`                | Discovered pages: title, url, source, draft/order/label |
| `markers_render_markdown` | `markdown`, `source_path?`    | Rendered HTML, outline, title, link edges               |
| `markers_site_graph`      | `content_dir?`                | Nodes/edges/stats without building                      |
| `markers_site_config`     | `content_dir?`                | Resolved config (title, basePath, theme, footer)        |

`content_dir`/`output_dir` default to `docs`/`dist`, matching the CLI.

## Not exposed

`serve` and `init` are intentionally absent — a long-running server and an
interactive scaffold don't fit a request/response tool model.
