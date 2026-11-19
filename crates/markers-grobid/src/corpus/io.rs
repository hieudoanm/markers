//! Corpus I/O helpers: TEI → `Work` records, per-work metadata JSON,
//! and the `.grobid/process.json` run manifest.

use std::path::{Path, PathBuf};

use crate::metadata::Metadata;
use crate::render::{extract_references, tei_root};
use crate::utils::text_content;

use super::Work;

/// Build a `Work` record from a parsed TEI document's metadata, abstract, and
/// reference list. `meta` may carry Crossref-enriched fields.
pub fn work_from_tei(
    tei_path: &Path,
    pdf_file: Option<&str>,
    md_file: Option<&str>,
    meta: &Metadata,
) -> anyhow::Result<Work> {
    let content = std::fs::read_to_string(tei_path)?;
    let doc = roxmltree::Document::parse(&content)?;
    let tei = tei_root(doc.root());

    let abstract_text = crate::render::descendant_named(tei, "teiHeader")
        .and_then(|h| crate::render::descendant_named(h, "profileDesc"))
        .and_then(|p| crate::render::descendant_named(p, "abstract"))
        .map(|a| text_content(&a))
        .filter(|s| !s.is_empty());

    let key = meta
        .doi
        .clone()
        .or_else(|| {
            pdf_file
                .map(|f| f.trim_end_matches(".pdf").to_string())
                .filter(|s| !s.is_empty())
        })
        .or_else(|| {
            tei_path
                .file_stem()
                .map(|s| s.to_string_lossy().trim_end_matches(".tei").to_string())
        })
        .unwrap_or_default();

    Ok(Work {
        key,
        doi: meta.doi.clone(),
        doi_link: meta.doi_link.clone(),
        title: meta.title.clone().unwrap_or_default(),
        authors: meta.authors.clone(),
        year: meta.year.clone(),
        journal: meta.journal.clone(),
        volume: meta.volume.clone(),
        issue: meta.issue.clone(),
        pages: meta.pages.clone(),
        kind: meta.kind.clone(),
        abstract_text: abstract_text.or_else(|| meta.abstract_text.clone()),
        cited_by: meta.cited_by,
        md_file: md_file.map(str::to_string),
        pdf_file: pdf_file.map(str::to_string),
        references: extract_references(tei),
    })
}

/// Write a work's full metadata (including references) as pretty JSON.
/// Returns the written path. Filename derives from the PDF stem so it lines
/// up with `md/<name>.md`.
pub fn write_metadata_file(dir: &Path, work: &Work) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let stem = work
        .pdf_file
        .as_deref()
        .map(|f| {
            Path::new(f)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| work.key.clone())
        })
        .unwrap_or_else(|| work.key.clone());
    let path = dir.join(format!("{stem}.json"));
    std::fs::write(&path, serde_json::to_string_pretty(work)?)?;
    Ok(path)
}

/// Per-file outcome recorded in the process manifest.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileResult {
    pub pdf: String,
    /// `converted` (Markdown written this run), `cached` (outputs reused),
    /// or `failed`.
    pub status: String,
}

/// One conversion run's metadata, appended to `.grobid/process.json`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RunRecord {
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u64,
    pub total: usize,
    pub converted: usize,
    pub cached: usize,
    pub failed: usize,
    pub enrich: bool,
    pub concurrency: usize,
    pub grobid_url: String,
    pub files: Vec<FileResult>,
}

/// Append a run record to the process manifest, keeping the last 50 runs.
pub fn append_process_manifest(path: &Path, record: &RunRecord) -> anyhow::Result<()> {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    struct Manifest {
        #[serde(default)]
        runs: Vec<serde_json::Value>,
    }
    let mut manifest = if path.exists() {
        let raw = std::fs::read_to_string(path)?;
        serde_json::from_str::<Manifest>(&raw).unwrap_or_default()
    } else {
        Manifest::default()
    };
    manifest.runs.push(serde_json::to_value(record)?);
    if manifest.runs.len() > 50 {
        let excess = manifest.runs.len() - 50;
        manifest.runs.drain(..excess);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(&manifest)?)?;
    Ok(())
}
