//! Document assembly: TEI document → Markdown string (frontmatter,
//! title/authors, abstract, body, references).

use crate::metadata::{Metadata, extract_metadata_doc, frontmatter};
use crate::render::references::render_reference;
use crate::render::tei::{author_name, descendant_named, descendants_named, tei_root};
use crate::render::{inline, render_children};

#[doc(hidden)]
pub fn convert_doc(
    doc: &roxmltree::Document,
    meta: Option<&Metadata>,
    fallback_title: &str,
) -> anyhow::Result<String> {
    let root = doc.root();

    // Find TEI element
    let tei = tei_root(root);

    // Title - prefer titleStmt title
    let title_from_meta = match meta {
        Some(m) => m.title.clone(),
        None => None,
    };
    let mut title = String::new();
    if let Some(t) = title_from_meta.as_ref() {
        if !t.is_empty() {
            title = t.clone();
        }
    }
    if title.is_empty() {
        if let Some(header) = descendant_named(tei, "teiHeader") {
            if let Some(file_desc) = descendant_named(header, "fileDesc") {
                if let Some(title_stmt) = descendant_named(file_desc, "titleStmt") {
                    let mut titles = descendants_named(title_stmt, "title");
                    titles.sort_by_key(|n| {
                        n.attribute("type")
                            .map(|t| if t.eq_ignore_ascii_case("main") { 0 } else { 1 })
                            .unwrap_or(1)
                    });
                    for t in titles {
                        let t2 = inline(t);
                        if !t2.is_empty() {
                            title = t2;
                            break;
                        }
                    }
                }
            }
        }
    }
    if title.is_empty() {
        title = fallback_title.to_string();
    }

    // Authors
    let mut authors: Vec<String> = Vec::new();
    if let Some(m) = meta {
        authors = m.authors.clone();
    }
    if authors.is_empty() {
        // Scope to teiHeader so authors cited in the reference list are excluded.
        let author_nodes = descendant_named(tei, "teiHeader")
            .map(|header| descendants_named(header, "author"))
            .unwrap_or_default();
        for author in author_nodes {
            let name = author_name(author);
            if !name.is_empty() {
                authors.push(name);
            }
        }
    }

    let abstract_node = descendant_named(tei, "teiHeader")
        .and_then(|header| descendant_named(header, "profileDesc"))
        .and_then(|profile| descendant_named(profile, "abstract"));
    let body_node = descendant_named(tei, "text").and_then(|text| descendant_named(text, "body"));

    let meta_resolved = match meta {
        Some(m) => m.clone(),
        None => extract_metadata_doc(doc),
    };
    let mut sections: Vec<String> = Vec::new();
    sections.push(frontmatter(&meta_resolved));
    sections.push(format!("# {}", title));
    if !authors.is_empty() {
        sections.push(format!("**Authors:** {}", authors.join(", ")));
    }
    if let Some(abs) = abstract_node {
        let abstract_md = render_children(abs, 2);
        if !abstract_md.is_empty() {
            sections.push("## Abstract".to_string());
            sections.push(abstract_md);
        }
    }
    if let Some(body) = body_node {
        let body_md = render_children(body, 2);
        if !body_md.is_empty() {
            sections.push("## Article".to_string());
            sections.push(body_md);
        }
    }

    // References: biblStruct/bibl entries inside listBibl.
    let mut references: Vec<String> = Vec::new();
    for list in descendants_named(tei, "listBibl") {
        for child in list.children() {
            if child.is_element() && matches!(child.tag_name().name(), "biblStruct" | "bibl") {
                let reference = render_reference(child);
                if !reference.is_empty() {
                    references.push(format!("- {reference}"));
                }
            }
        }
    }
    if !references.is_empty() {
        sections.push("## References".to_string());
        sections.push(references.join("\n"));
    }

    let result = sections.join("\n\n").trim().to_string();
    Ok(format!("{}\n", result))
}

/// Convert a TEI XML file to Markdown.
pub fn convert_tei_to_md(
    input_path: &std::path::Path,
    meta: Option<&Metadata>,
) -> anyhow::Result<String> {
    let content = std::fs::read_to_string(input_path)?;
    let doc = roxmltree::Document::parse(&content)?;
    let fallback_title = input_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .trim_end_matches(".tei")
        .to_string();
    convert_doc(&doc, meta, &fallback_title)
}

/// Convert a TEI XML string to Markdown.
pub fn convert_tei_xml(xml: &str) -> anyhow::Result<String> {
    let doc = roxmltree::Document::parse(xml)?;
    convert_doc(&doc, None, "")
}
