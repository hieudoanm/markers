//! TEI `teiHeader` → `Metadata` extraction.

use roxmltree::Node;

use super::Metadata;
use crate::render::{author_name, descendant_named, descendants_named, inline, tei_root};
use crate::utils::text_content;

/// Extract metadata from a parsed TEI document. Only the `teiHeader`
/// subtree is consulted so reference-list authors/titles are excluded.
pub fn extract_metadata_doc(doc: &roxmltree::Document) -> Metadata {
    let mut meta = Metadata::default();
    let tei = tei_root(doc.root());
    if tei.tag_name().name() != "TEI" {
        return meta;
    }
    let Some(header) = descendant_named(tei, "teiHeader") else {
        return meta;
    };

    // Title from fileDesc/titleStmt/title (prefer type="main" if present).
    if let Some(file_desc) = descendant_named(header, "fileDesc") {
        if let Some(title_stmt) = descendant_named(file_desc, "titleStmt") {
            let mut titles: Vec<Node> = descendants_named(title_stmt, "title");
            titles.sort_by_key(|n| {
                n.attribute("type")
                    .map(|t| if t.eq_ignore_ascii_case("main") { 0 } else { 1 })
                    .unwrap_or(1)
            });
            for node in titles {
                let t = inline(node);
                if !t.is_empty() {
                    meta.title = Some(t);
                    break;
                }
            }
        }
    }
    // Authors: only within the header so reference-list authors are excluded.
    let author_nodes = descendants_named(header, "author");
    for author in author_nodes {
        let name = author_name(author);
        if !name.is_empty() {
            meta.authors.push(name);
        }
    }

    // Journal/monograph metadata from sourceDesc biblStruct/monogr.
    if let Some(source_desc) = descendant_named(header, "sourceDesc") {
        if let Some(bibl_struct) = descendant_named(source_desc, "biblStruct") {
            if let Some(monogr) = descendant_named(bibl_struct, "monogr") {
                for title in descendants_named(monogr, "title") {
                    if title
                        .attribute("level")
                        .is_some_and(|l| l.eq_ignore_ascii_case("j"))
                    {
                        let journal = inline(title);
                        if !journal.is_empty() {
                            meta.journal = Some(journal);
                            break;
                        }
                    }
                }
                if let Some(imprint) = descendant_named(monogr, "imprint") {
                    for date in descendants_named(imprint, "date") {
                        let when = date
                            .attribute("when")
                            .or_else(|| date.attribute("notBefore"))
                            .unwrap_or("");
                        if when.len() >= 4 {
                            meta.year = Some(when[..4].to_string());
                            break;
                        }
                    }
                    for scope in descendants_named(imprint, "biblScope") {
                        let unit = scope.attribute("unit").unwrap_or("");
                        let value = text_content(&scope);
                        if value.is_empty() {
                            continue;
                        }
                        match unit {
                            "volume" => meta.volume = Some(value),
                            "issue" => meta.issue = Some(value),
                            "page" => meta.pages = Some(value),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // DOI from idno[@type='DOI'] anywhere in the header.
    for idno in descendants_named(header, "idno") {
        let is_doi = idno
            .attribute("type")
            .is_some_and(|t| t.eq_ignore_ascii_case("doi"));
        if !is_doi {
            continue;
        }
        let doi = text_content(&idno);
        if !doi.is_empty() {
            meta.doi_link = Some(if doi.starts_with("http") {
                doi.clone()
            } else {
                format!("https://doi.org/{doi}")
            });
            meta.doi = Some(doi);
            break;
        }
    }

    meta
}
