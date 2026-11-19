//! Bibliography parsing: `biblStruct`/`bibl` entries → `ParsedReference`.

use roxmltree::Node;

use super::inline::inline;
use super::tei::{author_name, descendant_named, descendants_named};
use crate::utils::text_content;

/// A parsed bibliography entry (biblStruct or bibl).
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ParsedReference {
    pub authors: Vec<String>,
    pub title: Option<String>,
    pub container: Option<String>,
    pub year: Option<String>,
    pub doi: Option<String>,
}

impl ParsedReference {
    /// Canonical display form: "Authors. Title. Container. Year", with the
    /// title linked to doi.org when a DOI is present.
    pub fn render(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if !self.authors.is_empty() {
            parts.push(self.authors.join(", "));
        }
        if let Some(title) = &self.title {
            match &self.doi {
                Some(doi) => parts.push(format!("[{title}](https://doi.org/{doi})")),
                None => parts.push(title.clone()),
            }
        }
        if let Some(container) = &self.container {
            parts.push(container.clone());
        }
        if let Some(year) = &self.year {
            parts.push(year.clone());
        }
        parts.join(". ")
    }
}

/// Extract the DOI from a `idno[@type='DOI']` descendant, if any.
fn reference_doi(entry: Node) -> Option<String> {
    descendants_named(entry, "idno")
        .iter()
        .filter(|n| {
            n.attribute("type")
                .is_some_and(|t| t.eq_ignore_ascii_case("doi"))
        })
        .map(|n| text_content(n))
        .find(|s| !s.is_empty())
}

/// Parse a single bibliography entry (biblStruct or bibl).
pub fn parse_reference(entry: Node) -> ParsedReference {
    let mut parsed = ParsedReference {
        doi: reference_doi(entry),
        ..Default::default()
    };

    // Authors and analytic (work) title.
    if let Some(analytic) = descendant_named(entry, "analytic") {
        parsed.authors = descendants_named(analytic, "author")
            .iter()
            .map(|a| author_name(*a))
            .filter(|s| !s.is_empty())
            .collect();
        if let Some(title) = descendant_named(analytic, "title") {
            let t = inline(title);
            if !t.is_empty() {
                parsed.title = Some(t);
            }
        }
    }

    // Container title (journal preferred via level="j") and imprint year.
    if let Some(monogr) = descendant_named(entry, "monogr") {
        let mut titles = descendants_named(monogr, "title");
        titles.sort_by_key(|n| {
            n.attribute("level")
                .map(|l| if l.eq_ignore_ascii_case("j") { 0 } else { 1 })
                .unwrap_or(1)
        });
        for t in titles {
            let s = inline(t);
            if !s.is_empty() {
                parsed.container = Some(s);
                break;
            }
        }
        if let Some(imprint) = descendant_named(monogr, "imprint") {
            if let Some(date) = descendant_named(imprint, "date") {
                let year = date
                    .attribute("when")
                    .filter(|w| w.chars().count() >= 4)
                    .map(|w| w.chars().take(4).collect::<String>())
                    .unwrap_or_else(|| text_content(&date));
                if !year.is_empty() {
                    parsed.year = Some(year);
                }
            }
        }
    }

    parsed
}

/// All bibliography entries (biblStruct/bibl) inside the document's listBibl
/// elements, in document order.
pub fn extract_references(tei: Node) -> Vec<ParsedReference> {
    let mut out = Vec::new();
    for list in descendants_named(tei, "listBibl") {
        for child in list.children() {
            if child.is_element() && matches!(child.tag_name().name(), "biblStruct" | "bibl") {
                out.push(parse_reference(child));
            }
        }
    }
    out
}

/// Render a single bibliography entry as a readable line, falling back to
/// flattened text when no structured fields are present.
pub(crate) fn render_reference(entry: Node) -> String {
    let parsed = parse_reference(entry);
    let rendered = parsed.render();
    if !rendered.is_empty() {
        return rendered;
    }
    text_content(&entry)
}
