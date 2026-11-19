//! `markers grobid`: PDF → Markdown via GROBID, plus corpus search/stats/
//! graph/MCP. Thin clap layer over `markers_grobid`.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{ArgAction, Args, Subcommand};

#[derive(Subcommand)]
pub enum GrobidCommand {
    /// Convert every PDF in ./pdf to TEI + Markdown (also the default when
    /// no subcommand is given).
    Convert,
    /// Generate the corpus dashboard (index.html) from the corpus DB.
    Graph {
        /// Corpus DB path (default: <data-dir>/.grobid/grobid.db).
        #[arg(long)]
        db: Option<PathBuf>,
        /// Output HTML path (default: <data-dir>/.grobid/index.html).
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Search the corpus index (title/authors/journal/DOI/abstract/type).
    Search {
        /// Search query (substring match).
        query: String,
        /// Maximum results.
        #[arg(long, default_value_t = 20)]
        limit: usize,
        /// Corpus DB path (default: <data-dir>/.grobid/grobid.db).
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Print corpus statistics (works, references, years, top authors/cited).
    Stats {
        /// Corpus DB path (default: <data-dir>/.grobid/grobid.db).
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Serve the corpus over MCP for AI agents.
    Mcp {
        /// Corpus DB path (default: <data-dir>/.grobid/grobid.db).
        #[arg(long)]
        db: Option<PathBuf>,
        /// Serve over streamable HTTP on this address instead of stdio.
        #[arg(long)]
        http: Option<String>,
        /// Bearer token required on HTTP requests (or MCP_AUTH_TOKEN).
        #[arg(long)]
        token: Option<String>,
    },
}

#[derive(Args)]
pub struct GrobidArgs {
    #[command(subcommand)]
    pub command: Option<GrobidCommand>,

    /// Enrich metadata via the Crossref API (adds type/cited_by to
    /// frontmatter, abstract/type/citations to the corpus index).
    /// Also enabled by GROBID_ENRICH=1. Only applies when converting.
    #[arg(long)]
    pub enrich: bool,

    /// Increase grobid log verbosity (-v debug, -vv trace). Also honors
    /// GROBID_LOG.
    #[arg(long, short = 'v', action = ArgAction::Count)]
    pub verbose: u8,
}

pub fn run(args: GrobidArgs) -> ExitCode {
    markers_grobid::utils::bump_level(args.verbose);
    let result = match args.command {
        Some(GrobidCommand::Graph { db, out }) => finish(markers_grobid::cli::graph_cmd(db, out)),
        Some(GrobidCommand::Search { query, limit, db }) => {
            finish(markers_grobid::cli::search_cmd(&query, limit, db))
        }
        Some(GrobidCommand::Stats { db }) => finish(markers_grobid::cli::stats_cmd(db)),
        Some(GrobidCommand::Mcp { db, http, token }) => finish(block_on(async {
            match http {
                Some(addr) => markers_grobid::mcp::serve_http(
                    db,
                    &addr,
                    token.or_else(|| std::env::var("MCP_AUTH_TOKEN").ok()),
                )
                .await
                .map(|_| 0),
                None => markers_grobid::mcp::serve(db).await.map(|_| 0),
            }
        })),
        Some(GrobidCommand::Convert) | None => finish(block_on(
            markers_grobid::cli::convert_pdf_to_md(args.enrich),
        )),
    };
    // Stop a locally spawned GROBID service (no-op for Docker/remote/external).
    markers_grobid::grobid_client::stop_local_service();
    result
}

fn finish<E: std::fmt::Display>(result: Result<i32, E>) -> ExitCode {
    match result {
        Ok(code) => ExitCode::from(code.clamp(0, 255) as u8),
        Err(e) => {
            eprintln!("markers grobid: {e}");
            ExitCode::FAILURE
        }
    }
}

fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime")
        .block_on(fut)
}
