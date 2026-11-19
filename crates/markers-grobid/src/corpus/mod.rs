//! Corpus index: a SQLite database of all converted works plus their
//! reference edges, searchable via `grobid search`/`stats`/`graph`.

mod graph;
mod io;
mod query;
mod stats;

use std::path::Path;

use rusqlite::Connection;

pub use graph::{GraphEdge, GraphNode, render_graph_html};
pub use io::{FileResult, RunRecord, append_process_manifest, work_from_tei, write_metadata_file};
pub use query::WorkRow;
pub use stats::{Stats, log_corpus_summary};

/// A single converted work plus its parsed references. Serializes to the
/// per-work metadata JSON written under `<data-dir>/metadata/`.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Work {
    /// Stable identity: DOI when present, otherwise the PDF file stem.
    pub key: String,
    pub doi: Option<String>,
    pub doi_link: Option<String>,
    pub title: String,
    pub authors: Vec<String>,
    pub year: Option<String>,
    pub journal: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub kind: Option<String>,
    pub abstract_text: Option<String>,
    pub cited_by: Option<i64>,
    pub md_file: Option<String>,
    pub pdf_file: Option<String>,
    pub references: Vec<crate::render::ParsedReference>,
}

impl Work {
    /// Number of references lacking both a title and a DOI (unresolved stubs).
    pub fn stub_refs(&self) -> usize {
        self.references
            .iter()
            .filter(|r| r.title.as_deref().unwrap_or("").is_empty() && r.doi.is_none())
            .count()
    }

    /// Number of references whose DOI equals this work's DOI (self-citations).
    pub fn self_refs(&self) -> usize {
        match &self.doi {
            Some(doi) => self
                .references
                .iter()
                .filter(|r| r.doi.as_deref() == Some(doi.as_str()))
                .count(),
            None => 0,
        }
    }
}

/// Open (creating if needed) the corpus database and run migrations.
pub fn open_db(path: &Path) -> anyhow::Result<CorpusDb> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS works(
           id INTEGER PRIMARY KEY,
           key TEXT NOT NULL UNIQUE,
           doi TEXT,
           title TEXT NOT NULL DEFAULT '',
           authors TEXT NOT NULL DEFAULT '',
           year TEXT NOT NULL DEFAULT '',
           journal TEXT NOT NULL DEFAULT '',
           kind TEXT NOT NULL DEFAULT '',
           abstract TEXT NOT NULL DEFAULT '',
           cited_by INTEGER,
           md_file TEXT NOT NULL DEFAULT '',
           pdf_file TEXT NOT NULL DEFAULT '',
           ref_count INTEGER NOT NULL DEFAULT 0,
           stub_refs INTEGER NOT NULL DEFAULT 0,
           self_refs INTEGER NOT NULL DEFAULT 0
         );
         CREATE INDEX IF NOT EXISTS idx_works_doi ON works(doi);
         CREATE TABLE IF NOT EXISTS refs(
           id INTEGER PRIMARY KEY,
           work_key TEXT NOT NULL,
           ref_doi TEXT,
           ref_title TEXT NOT NULL DEFAULT '',
           ref_container TEXT NOT NULL DEFAULT '',
           ref_year TEXT NOT NULL DEFAULT '',
           is_stub INTEGER NOT NULL DEFAULT 0,
           is_self INTEGER NOT NULL DEFAULT 0
         );
         CREATE INDEX IF NOT EXISTS idx_refs_work ON refs(work_key);
         CREATE INDEX IF NOT EXISTS idx_refs_doi ON refs(ref_doi);",
    )?;
    Ok(CorpusDb { conn })
}

pub struct CorpusDb {
    conn: Connection,
}
