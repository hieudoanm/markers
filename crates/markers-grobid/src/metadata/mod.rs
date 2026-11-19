//! Metadata extraction from TEI XML.

mod extract;

use crate::config::META_FIELDS;
use crate::utils::yaml_string;

pub use extract::extract_metadata_doc;

/// Build YAML frontmatter string from a metadata map. Empty fields are
/// omitted so consumers don't have to distinguish "" from "absent".
pub fn frontmatter(meta: &Metadata) -> String {
    let mut lines: Vec<String> = vec!["---".to_string()];
    for field in META_FIELDS {
        let value = match field.to_string().as_str() {
            "authors" => meta.authors.join(", "),
            "year" => meta.year.clone().unwrap_or_default(),
            "title" => meta.title.clone().unwrap_or_default(),
            "journal" => meta.journal.clone().unwrap_or_default(),
            "volume" => meta.volume.clone().unwrap_or_default(),
            "issue" => meta.issue.clone().unwrap_or_default(),
            "pages" => meta.pages.clone().unwrap_or_default(),
            "doi" => meta.doi.clone().unwrap_or_default(),
            "doi_link" => meta.doi_link.clone().unwrap_or_default(),
            "type" => meta.kind.clone().unwrap_or_default(),
            "cited_by" => meta.cited_by.map(|n| n.to_string()).unwrap_or_default(),
            _ => String::new(),
        };
        if value.is_empty() {
            continue;
        }
        if *field == "year" && value.chars().all(|c| c.is_ascii_digit()) {
            lines.push(format!("year: {value}"));
        } else {
            lines.push(format!("{}: {}", field, yaml_string(&value)));
        }
    }
    lines.push("---".to_string());
    lines.join("\n")
}

/// Metadata extracted from a TEI document.
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub year: Option<String>,
    pub journal: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub doi: Option<String>,
    pub doi_link: Option<String>,
    /// Work type (e.g. `journal-article`); only set via Crossref enrichment.
    pub kind: Option<String>,
    /// Citation count from Crossref (`is-referenced-by-count`).
    pub cited_by: Option<i64>,
    /// Abstract plain text; corpus index only, not emitted in frontmatter.
    pub abstract_text: Option<String>,
}

/// Extract metadata from a TEI XML file.
pub fn extract_metadata_from_tei(tei_path: &std::path::Path) -> anyhow::Result<Metadata> {
    let content = std::fs::read_to_string(tei_path)?;
    let doc = roxmltree::Document::parse(&content)?;
    Ok(extract_metadata_doc(&doc))
}
