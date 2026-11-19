//! Corpus writes and search: work upserts, unified LIKE search, and the
//! portable JSON index export.

use std::path::Path;

use rusqlite::params;
use serde::Serialize;

use super::{CorpusDb, Work};

impl CorpusDb {
    /// Insert or replace a work and rewrite its reference edges.
    pub fn upsert_work(&mut self, work: &Work) -> anyhow::Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO works(key, doi, title, authors, year, journal, kind, abstract,
                                cited_by, md_file, pdf_file, ref_count, stub_refs, self_refs)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
             ON CONFLICT(key) DO UPDATE SET
               doi=excluded.doi, title=excluded.title, authors=excluded.authors,
               year=excluded.year, journal=excluded.journal, kind=excluded.kind,
               abstract=excluded.abstract, cited_by=excluded.cited_by,
               md_file=excluded.md_file, pdf_file=excluded.pdf_file,
               ref_count=excluded.ref_count, stub_refs=excluded.stub_refs,
               self_refs=excluded.self_refs",
            params![
                work.key.clone(),
                work.doi.clone(),
                work.title.clone(),
                work.authors.join(", "),
                work.year.clone().unwrap_or_default(),
                work.journal.clone().unwrap_or_default(),
                work.kind.clone().unwrap_or_default(),
                work.abstract_text.clone().unwrap_or_default(),
                work.cited_by,
                work.md_file.clone().unwrap_or_default(),
                work.pdf_file.clone().unwrap_or_default(),
                work.references.len() as i64,
                work.stub_refs() as i64,
                work.self_refs() as i64,
            ],
        )?;
        tx.execute("DELETE FROM refs WHERE work_key = ?1", params![work.key])?;
        for r in &work.references {
            let is_stub = r.title.as_deref().unwrap_or("").is_empty() && r.doi.is_none();
            let is_self = work
                .doi
                .as_deref()
                .is_some_and(|d| r.doi.as_deref() == Some(d));
            tx.execute(
                "INSERT INTO refs(work_key, ref_doi, ref_title, ref_container, ref_year,
                                        is_stub, is_self)
                 VALUES(?1,?2,?3,?4,?5,?6,?7)",
                params![
                    work.key,
                    r.doi,
                    r.title.clone().unwrap_or_default(),
                    r.container.clone().unwrap_or_default(),
                    r.year.clone().unwrap_or_default(),
                    is_stub as i64,
                    is_self as i64,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Case-insensitive substring search across title/authors/journal/doi/
    /// abstract/kind — the same unified-filter semantics as the doi app.
    pub fn search(&self, query: &str, limit: usize) -> anyhow::Result<Vec<WorkRow>> {
        let pattern = format!("%{}%", escape_like(query));
        let mut stmt = self.conn.prepare(
            "SELECT key, doi, title, authors, year, journal, kind, cited_by, md_file
             FROM works
             WHERE title LIKE ?1 ESCAPE '\\'
                OR authors LIKE ?1 ESCAPE '\\'
                OR journal LIKE ?1 ESCAPE '\\'
                OR doi LIKE ?1 ESCAPE '\\'
                OR abstract LIKE ?1 ESCAPE '\\'
                OR kind LIKE ?1 ESCAPE '\\'
             ORDER BY year DESC, title
             LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![pattern, limit as i64], WorkRow::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Fetch one work by exact `key` (DOI or pdf stem).
    pub fn get_work(&self, key: &str) -> anyhow::Result<Option<WorkRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT key, doi, title, authors, year, journal, kind, cited_by, md_file
             FROM works WHERE key = ?1",
        )?;
        let mut rows = stmt.query_map(params![key], WorkRow::from_row)?;
        Ok(rows.next().transpose()?)
    }

    /// Write a portable JSON search index (works only, no edges).
    pub fn export_index(&self, path: &Path) -> anyhow::Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT doi, title, authors, year, journal, kind, cited_by, md_file
             FROM works ORDER BY year DESC, title",
        )?;
        let items: Vec<IndexEntry> = stmt
            .query_map([], |r| {
                Ok(IndexEntry {
                    doi: r.get(0)?,
                    title: r.get(1)?,
                    authors: r
                        .get::<_, String>(2)?
                        .split(", ")
                        .map(str::to_string)
                        .collect(),
                    year: r.get::<_, String>(3)?,
                    journal: r.get::<_, String>(4)?,
                    kind: r.get::<_, String>(5)?,
                    cited_by: r.get(6)?,
                    md_file: r.get::<_, String>(7)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(&items)?)?;
        Ok(())
    }
}

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// A work row returned by search.
#[derive(Debug)]
pub struct WorkRow {
    pub key: String,
    pub doi: Option<String>,
    pub title: String,
    pub authors: String,
    pub year: String,
    pub journal: String,
    pub kind: String,
    pub cited_by: Option<i64>,
    pub md_file: String,
}

impl WorkRow {
    fn from_row(r: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            key: r.get(0)?,
            doi: r.get(1)?,
            title: r.get(2)?,
            authors: r.get(3)?,
            year: r.get(4)?,
            journal: r.get(5)?,
            kind: r.get(6)?,
            cited_by: r.get(7)?,
            md_file: r.get(8)?,
        })
    }
}

#[derive(Serialize)]
struct IndexEntry {
    doi: Option<String>,
    title: String,
    authors: Vec<String>,
    year: String,
    journal: String,
    kind: String,
    cited_by: Option<i64>,
    md_file: String,
}
