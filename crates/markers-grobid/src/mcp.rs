//! MCP server over stdio (`markers grobid mcp`) exposing the local corpus:
//! stats, search, work listing, and graph generation.

use std::path::PathBuf;

use crate::{corpus, paths};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, ContentBlock, ErrorData as McpError, ListResourcesResult,
    PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResponse, ReadResourceResult,
    Resource, ResourceContents, ServerCapabilities, ServerConfig,
};
use rmcp::schemars::JsonSchema;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use serde::Deserialize;

#[derive(Clone)]
pub struct GrobidMcp {
    db_path: PathBuf,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchParams {
    /// Substring query matched against title, authors, journal, DOI,
    /// abstract, and type.
    pub query: String,
    /// Maximum results (default 20).
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LimitParams {
    /// Maximum results (default 50).
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GraphParams {
    /// Output HTML path (default: <data-dir>/.grobid/index.html).
    #[serde(default)]
    pub out: Option<String>,
}

fn internal_error(e: impl std::fmt::Display) -> McpError {
    McpError::internal_error(e.to_string(), None)
}

fn pct_encode(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

fn pct_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn work_uri(key: &str) -> String {
    format!("grobid://works/{}", pct_encode(key))
}

fn json_result<T: serde::Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(value).map_err(internal_error)?;
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

#[tool_router]
impl GrobidMcp {
    #[tool(description = "Corpus statistics: works, references, DOI coverage, \
            year span, distinct authors, and top in/out-degree rankings.")]
    async fn grobid_stats(&self) -> Result<CallToolResult, McpError> {
        let db = corpus::open_db(&self.db_path).map_err(internal_error)?;
        let s = db.stats().map_err(internal_error)?;
        json_result(&serde_json::json!({
            "works": s.works,
            "titled": s.titled,
            "untitled": s.works - s.titled,
            "references": s.references,
            "doi_refs": s.doi_refs,
            "stub_refs": s.stub_refs,
            "self_refs": s.self_refs,
            "distinct_authors": s.distinct_authors,
            "min_year": s.min_year,
            "max_year": s.max_year,
            "year_hist": s.year_hist,
            "top_authors": s.top_authors,
            "top_in_cited": s.top_in_cited,
            "top_citing": s.top_citing,
            "top_cited_crossref": s.top_cited,
        }))
    }

    #[tool(description = "Search the corpus index. Substring match across \
            title, authors, journal, DOI, abstract, and type.")]
    async fn grobid_search(
        &self,
        Parameters(p): Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let db = corpus::open_db(&self.db_path).map_err(internal_error)?;
        let rows = db
            .search(&p.query, p.limit.unwrap_or(20))
            .map_err(internal_error)?;
        json_result(
            &rows
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "key": r.key,
                        "doi": r.doi,
                        "title": r.title,
                        "authors": r.authors,
                        "year": r.year,
                        "journal": r.journal,
                        "kind": r.kind,
                        "cited_by": r.cited_by,
                        "md_file": r.md_file,
                    })
                })
                .collect::<Vec<_>>(),
        )
    }

    #[tool(description = "List works in the corpus, most recent metadata first.")]
    async fn grobid_works(
        &self,
        Parameters(p): Parameters<LimitParams>,
    ) -> Result<CallToolResult, McpError> {
        let db = corpus::open_db(&self.db_path).map_err(internal_error)?;
        let rows = db
            .search("", p.limit.unwrap_or(50))
            .map_err(internal_error)?;
        json_result(
            &rows
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "key": r.key,
                        "doi": r.doi,
                        "title": r.title,
                        "authors": r.authors,
                        "year": r.year,
                        "journal": r.journal,
                        "kind": r.kind,
                    })
                })
                .collect::<Vec<_>>(),
        )
    }

    #[tool(description = "Generate the interactive citation-graph dashboard \
            (self-contained HTML with d3-force, stats, and unified search) \
            and return its path.")]
    async fn grobid_graph(
        &self,
        Parameters(p): Parameters<GraphParams>,
    ) -> Result<CallToolResult, McpError> {
        let db = corpus::open_db(&self.db_path).map_err(internal_error)?;
        let html = corpus::render_graph_html(&db).map_err(internal_error)?;
        let out = p.out.map(PathBuf::from).unwrap_or_else(paths::graph_path);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(internal_error)?;
        }
        std::fs::write(&out, html).map_err(internal_error)?;
        json_result(&serde_json::json!({ "path": out.display().to_string() }))
    }
}

#[tool_handler]
impl ServerHandler for GrobidMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_instructions(
            "Local GROBID corpus tools: statistics, full-text search over \
             converted papers, work listing, and citation-graph dashboard \
             generation. Corpus works are also exposed as resources under \
             grobid://works/<key> (converted Markdown when available). \
             Requires a corpus DB built by running `markers grobid`."
                .to_string(),
        )
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let db = corpus::open_db(&self.db_path).map_err(internal_error)?;
        let rows = db.search("", 1000).map_err(internal_error)?;
        let mut resources = vec![
            Resource::new("grobid://stats", "corpus-stats")
                .with_title("Corpus Statistics")
                .with_description("Work/reference counts, year span, and author totals as JSON.")
                .with_mime_type("application/json"),
        ];
        resources.extend(rows.iter().map(|r| {
            let mut res = Resource::new(work_uri(&r.key), r.key.clone())
                .with_title(if r.title.is_empty() {
                    r.key.clone()
                } else {
                    r.title.clone()
                })
                .with_description(format!(
                    "{} — {}{}",
                    r.authors,
                    r.journal,
                    r.year
                        .split(',')
                        .next()
                        .map(|y| format!(" ({})", y.trim()))
                        .unwrap_or_default()
                ));
            if !r.md_file.is_empty() {
                res = res.with_mime_type("text/markdown");
            }
            res
        }));
        Ok(ListResourcesResult::with_all_items(resources))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        let uri = request.uri;
        if uri == "grobid://stats" {
            let db = corpus::open_db(&self.db_path).map_err(internal_error)?;
            let s = db.stats().map_err(internal_error)?;
            let text = serde_json::to_string_pretty(&serde_json::json!({
                "works": s.works,
                "references": s.references,
                "distinct_authors": s.distinct_authors,
                "min_year": s.min_year,
                "max_year": s.max_year,
            }))
            .map_err(internal_error)?;
            return Ok(ReadResourceResult::new(vec![
                ResourceContents::text(text, uri).with_mime_type("application/json"),
            ])
            .into());
        }
        let key = uri
            .strip_prefix("grobid://works/")
            .ok_or_else(|| McpError::invalid_params("unknown resource URI", None))?;
        let key = pct_decode(key);
        let db = corpus::open_db(&self.db_path).map_err(internal_error)?;
        let work = db
            .get_work(&key)
            .map_err(internal_error)?
            .ok_or_else(|| McpError::invalid_params(format!("no such work: {key}"), None))?;

        let md_dir = paths::dirs().2;
        let md_path = md_dir.join(&work.md_file);
        let (text, mime) = if !work.md_file.is_empty() && md_path.is_file() {
            (
                std::fs::read_to_string(&md_path).map_err(internal_error)?,
                "text/markdown",
            )
        } else {
            (
                serde_json::to_string_pretty(&serde_json::json!({
                    "key": work.key,
                    "doi": work.doi,
                    "title": work.title,
                    "authors": work.authors,
                    "year": work.year,
                    "journal": work.journal,
                    "kind": work.kind,
                    "cited_by": work.cited_by,
                    "md_file": work.md_file,
                }))
                .map_err(internal_error)?,
                "application/json",
            )
        };
        Ok(
            ReadResourceResult::new(vec![ResourceContents::text(text, uri).with_mime_type(mime)])
                .into(),
        )
    }
}

/// Serve MCP over stdio until the client disconnects.
pub async fn serve(db_arg: Option<PathBuf>) -> anyhow::Result<()> {
    let db_path = db_arg.unwrap_or_else(paths::db_path);
    let service = GrobidMcp { db_path };
    let running = service
        .serve((tokio::io::stdin(), tokio::io::stdout()))
        .await?;
    running.waiting().await?;
    Ok(())
}

/// Serve MCP over streamable HTTP on `addr` (POST /mcp, plus GET SSE and
/// DELETE session management). When `token` is set, requests must carry
/// `Authorization: Bearer <token>`.
pub async fn serve_http(
    db_arg: Option<PathBuf>,
    addr: &str,
    token: Option<String>,
) -> anyhow::Result<()> {
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    };

    let db_path = db_arg.unwrap_or_else(paths::db_path);
    let service = std::sync::Arc::new(StreamableHttpService::new(
        move || {
            Ok(GrobidMcp {
                db_path: db_path.clone(),
            })
        },
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
