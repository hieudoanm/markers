//! markers-mcp: exposes markers as an MCP server over stdio, so MCP clients
//! can build sites, inspect pages, render Markdown and query the link graph
//! without shelling out to the CLI.
//!
//! ```text
//! markers-mcp    # speaks MCP on stdin/stdout; launch it from an MCP client
//! ```

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorData;
use rmcp::schemars::{self, JsonSchema};
use rmcp::transport::stdio;
use rmcp::{ServiceExt, tool, tool_router};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Clone)]
struct Markers;

#[derive(Debug, Deserialize, JsonSchema)]
struct DirParams {
    /// Directory of Markdown content (default: ./docs).
    content_dir: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct BuildParams {
    /// Directory of Markdown content (default: ./docs).
    content_dir: Option<String>,
    /// Directory the static site is written to (default: ./dist).
    output_dir: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RenderParams {
    /// Markdown source to render.
    markdown: String,
    /// Page path used for link resolution context, e.g. `guide/install.md`.
    source: Option<String>,
}

#[tool_router(server_handler)]
impl Markers {
    /// Build a static site from a directory of Markdown files.
    #[tool(
        description = "Build a static site: render every Markdown page, copy assets, emit the link graph"
    )]
    fn markers_build(
        &self,
        Parameters(params): Parameters<BuildParams>,
    ) -> Result<String, ErrorData> {
        let content = params.content_dir.unwrap_or_else(|| "docs".into());
        let output = params.output_dir.unwrap_or_else(|| "dist".into());
        let stats = markers_build::build(&content, &output).map_err(internal)?;
        to_json(&json!({
            "pages": stats.pages,
            "assets": stats.assets,
            "output_dir": output,
        }))
    }

    /// List the pages discovered under a content directory.
    #[tool(
        description = "List pages (title, URL, source path, draft/order metadata) in a Markdown directory"
    )]
    fn markers_list_pages(
        &self,
        Parameters(params): Parameters<DirParams>,
    ) -> Result<String, ErrorData> {
        let dir = params.content_dir.unwrap_or_else(|| "docs".into());
        let content = markers_site::site::discover(&dir).map_err(internal)?;
        let pages: Vec<Value> = content
            .pages
            .iter()
            .map(|p| {
                json!({
                    "source": p.source,
                    "title": p.title,
                    "url": p.url,
                    "draft": p.draft,
                    "order": p.order,
                    "label": p.label,
                })
            })
            .collect();
        to_json(&json!({
            "content_dir": dir,
            "pages": pages,
            "assets": content.assets.len(),
        }))
    }

    /// Render a Markdown string to HTML without site context.
    #[tool(
        description = "Render a Markdown string to HTML, returning the body HTML, page outline and detected title"
    )]
    fn markers_render_markdown(
        &self,
        Parameters(params): Parameters<RenderParams>,
    ) -> Result<String, ErrorData> {
        let source = params.source.unwrap_or_else(|| "page.md".into());
        let rendered = markers_markdown::render(&source, &params.markdown, None);
        let outline: Vec<Value> = rendered
            .outline
            .iter()
            .map(|h| json!({"level": h.level, "id": h.id, "text": h.text}))
            .collect();
        to_json(&json!({
            "html": rendered.html,
            "title": rendered.title,
            "outline": outline,
            "edges": rendered.edges,
        }))
    }

    /// Scan a content directory and return its page-link graph.
    #[tool(
        description = "Build the page-link graph for a Markdown directory: nodes are pages, edges are internal Markdown links"
    )]
    fn markers_site_graph(
        &self,
        Parameters(params): Parameters<DirParams>,
    ) -> Result<String, ErrorData> {
        let dir = params.content_dir.unwrap_or_else(|| "docs".into());
        let graph = markers_graph::scan(&dir).map_err(internal)?;
        let mut payload = markers_graph::to_json(&graph);
        payload["stats"] = json!({
            "nodes": graph.stats.nodes,
            "edges": graph.stats.edges,
            "broken": graph.stats.broken,
        });
        to_json(&payload)
    }

    /// Read the effective site configuration for a content directory.
    #[tool(
        description = "Read site configuration (title, language, basePath, theme, footer) resolved from index frontmatter or markers.yaml"
    )]
    fn markers_site_config(
        &self,
        Parameters(params): Parameters<DirParams>,
    ) -> Result<String, ErrorData> {
        let dir = params.content_dir.unwrap_or_else(|| "docs".into());
        let config = markers_site::config::load_for_dir(&dir).map_err(internal)?;
        to_json(&json!({
            "title": config.title,
            "language": config.language,
            "basePath": config.base_path,
            "theme": config.theme,
            "footer": config.footer,
        }))
    }
}

fn internal(message: String) -> ErrorData {
    ErrorData::internal_error(message, None)
}

fn to_json(value: &Value) -> Result<String, ErrorData> {
    serde_json::to_string_pretty(value).map_err(|e| internal(e.to_string()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1).cloned())
    };
    if let Some(addr) = flag("--http") {
        let token = flag("--token").or_else(|| std::env::var("MCP_AUTH_TOKEN").ok());
        return serve_http(&addr, token).await;
    }
    let service = Markers.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

/// Serve MCP over streamable HTTP on `addr` (POST /mcp, plus GET SSE and
/// DELETE session management). When `token` is set, requests must carry
/// `Authorization: Bearer <token>`.
async fn serve_http(addr: &str, token: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    };

    let service = std::sync::Arc::new(StreamableHttpService::new(
        || Ok(Markers),
        std::sync::Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    ));
    let svc = std::sync::Arc::clone(&service);
    let mut app = axum::Router::new().route(
        "/mcp",
        axum::routing::any(move |req: axum::extract::Request| {
            let svc = std::sync::Arc::clone(&svc);
            async move { svc.handle(req).await }
        }),
    );
    if let Some(expected) = token {
        app = app.layer(axum::middleware::from_fn_with_state(
            expected,
            |axum::extract::State(expected): axum::extract::State<String>,
             req: axum::extract::Request,
             next: axum::middleware::Next| async move {
                let ok = req
                    .headers()
                    .get(axum::http::header::AUTHORIZATION)
                    .and_then(|v| v.to_str().ok())
                    .is_some_and(|v| v == format!("Bearer {expected}"));
                if ok {
                    Ok(next.run(req).await)
                } else {
                    Err(axum::http::StatusCode::UNAUTHORIZED)
                }
            },
        ));
    }
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("[mcp] streamable-http listening on http://{addr}/mcp");
    axum::serve(listener, app).await?;
    Ok(())
}
