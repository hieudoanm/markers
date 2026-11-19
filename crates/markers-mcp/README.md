# markers-mcp

`markers-mcp` — a stdio [Model Context Protocol](https://modelcontextprotocol.io)
server (built on `rmcp`) that exposes markers' operations as tools so MCP
clients can build sites and inspect content programmatically.

Part of the [markers](../../README.md) workspace; see
[MCP server](../../packages/landing/docs/mcp.md) for client configuration.

## Tools

| Tool                      | Purpose                                                        |
| ------------------------- | -------------------------------------------------------------- |
| `markers_build`           | Build a site (`content_dir`, `output_dir`) → page/asset counts |
| `markers_list_pages`      | Discover pages → title, URL, source, draft/order/label         |
| `markers_render_markdown` | Render a Markdown string → HTML, outline, title, edges         |
| `markers_site_graph`      | Scan a directory → nodes/edges/stats JSON                      |
| `markers_site_config`     | Resolved site config (title, basePath, theme, footer)          |

Tool names carry the `markers_` prefix to avoid collisions with other MCP
servers. All results are pretty-printed JSON; failures return
`ErrorData::internal_error`.

## Usage

```bash
cargo run -p markers-mcp           # stdio server — point an MCP client at it
```

```json
{ "mcpServers": { "markers": { "command": "markers-mcp" } } }
```
