//! Aggregate corpus statistics: counts, year histogram, and author /
//! citation rankings.

use super::CorpusDb;
use crate::utils::log;

/// Aggregate corpus statistics.
pub struct Stats {
    pub works: i64,
    pub titled: i64,
    pub references: i64,
    pub doi_refs: i64,
    pub stub_refs: i64,
    pub self_refs: i64,
    pub min_year: String,
    pub max_year: String,
    pub year_hist: Vec<(String, i64)>,
    pub top_authors: Vec<(String, i64)>,
    pub top_cited: Vec<(String, Option<String>, i64)>,
    /// Count of distinct author strings (doi app parity).
    pub distinct_authors: i64,
    /// Works ranked by in-degree: how many corpus works cite them.
    pub top_in_cited: Vec<(String, Option<String>, i64)>,
    /// Works ranked by out-degree: how many DOI-resolved refs they cite.
    pub top_citing: Vec<(String, Option<String>, i64)>,
}

impl CorpusDb {
    pub fn stats(&self) -> anyhow::Result<Stats> {
        let mut stmt = self.conn.prepare(
            "SELECT year, COUNT(*) FROM works WHERE year != '' GROUP BY year ORDER BY year",
        )?;
        let year_hist: Vec<(String, i64)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<_, _>>()?;

        let scalar =
            |sql: &str| -> anyhow::Result<i64> { Ok(self.conn.query_row(sql, [], |r| r.get(0))?) };

        // Per-author counts: works.authors is a joined string, so split on
        // the separators GROBID emits (comma, semicolon, pipe, "and") and
        // count individuals case-insensitively.
        let author_rows = {
            let mut stmt = self
                .conn
                .prepare("SELECT authors FROM works WHERE authors != ''")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };
        let mut author_counts: std::collections::HashMap<String, (String, i64)> =
            std::collections::HashMap::new();
        for row in &author_rows {
            for name in split_authors(row) {
                let key = name.to_lowercase();
                let entry = author_counts
                    .entry(key)
                    .or_insert_with(|| (name.clone(), 0));
                entry.1 += 1;
            }
        }
        let distinct_authors = author_counts.len() as i64;
        let mut top_authors: Vec<(String, i64)> = author_counts.into_values().collect();
        top_authors.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        top_authors.truncate(10);

        let top_cited = {
            let mut stmt = self.conn.prepare(
                "SELECT title, doi, cited_by FROM works
                 WHERE cited_by IS NOT NULL AND title != ''
                 ORDER BY cited_by DESC LIMIT 10",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        // Corpus-derived rankings (doi app parity): in-degree = works citing
        // this work, out-degree = DOI-resolved references this work cites.
        // Self-citations and untitled stub works are excluded.
        // In-degree over every cited DOI (corpus works AND external stubs):
        // which references does this corpus cite most? Titles resolve via
        // LEFT JOIN; external-only DOIs fall back to the bare DOI.
        let top_in_cited = {
            let mut stmt = self.conn.prepare(
                "SELECT COALESCE(w.title, ''), r.ref_doi, COUNT(*) AS c
                 FROM refs r LEFT JOIN works w ON w.doi = r.ref_doi
                 WHERE r.is_self = 0 AND r.ref_doi IS NOT NULL AND r.ref_doi != ''
                 GROUP BY r.ref_doi ORDER BY c DESC LIMIT 10",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let top_citing = {
            let mut stmt = self.conn.prepare(
                "SELECT w.title, w.doi, COUNT(*) AS c
                 FROM refs r JOIN works w ON w.key = r.work_key
                 WHERE r.ref_doi IS NOT NULL AND r.ref_doi != '' AND w.title != ''
                 GROUP BY w.id ORDER BY c DESC LIMIT 10",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let (min_year, max_year) = year_hist
            .first()
            .zip(year_hist.last())
            .map(|(a, b)| (a.0.clone(), b.0.clone()))
            .unwrap_or_default();

        Ok(Stats {
            works: scalar("SELECT COUNT(*) FROM works")?,
            titled: scalar("SELECT COUNT(*) FROM works WHERE title != ''")?,
            references: scalar("SELECT COUNT(*) FROM refs")?,
            doi_refs: scalar(
                "SELECT COUNT(*) FROM refs WHERE ref_doi IS NOT NULL AND ref_doi != ''",
            )?,
            stub_refs: {
                let mut stmt = self
                    .conn
                    .prepare("SELECT COALESCE(SUM(stub_refs),0) FROM works")?;
                stmt.query_row([], |r| r.get(0))?
            },
            self_refs: {
                let mut stmt = self
                    .conn
                    .prepare("SELECT COALESCE(SUM(self_refs),0) FROM works")?;
                stmt.query_row([], |r| r.get(0))?
            },
            min_year,
            max_year,
            year_hist,
            top_authors,
            top_cited,
            distinct_authors,
            top_in_cited,
            top_citing,
        })
    }
}

/// Split a joined author string into individual names. GROBID output uses
/// commas, semicolons, pipes, and "and" as separators; names themselves
/// never contain those.
fn split_authors(authors: &str) -> Vec<String> {
    authors
        .split([',', ';', '|'])
        .flat_map(|part| part.split(" and "))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect()
}

/// Log a one-line corpus summary after an upsert pass.
pub fn log_corpus_summary(db: &CorpusDb) {
    match db.stats() {
        Ok(s) => log().info(&format!(
            "Corpus: {} works, {} reference edges ({} with DOI)",
            s.works, s.references, s.doi_refs
        )),
        Err(e) => log().warn(&format!("Could not read corpus stats: {e}")),
    }
}
