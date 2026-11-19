//! Inline (span-level) TEI → Markdown rendering.

use roxmltree::Node;

use super::tei::collapse_spaces;

/// Render an element's inline content: text plus `hi`/`ref`/`lb` children.
pub fn inline(element: Node) -> String {
    let mut parts: Vec<String> = Vec::new();
    // Don't use element.text() as it concatenates all descendants causing duplication
    // when we also process children
    for child in element.children() {
        if child.is_element() {
            let tag = child.tag_name().name();
            if tag == "hi" {
                let content = inline(child);
                let rend = child.attribute("rend").unwrap_or("").to_lowercase();
                if rend.contains("italic") {
                    parts.push(format!("*{}*", content));
                } else if rend.contains("bold") {
                    parts.push(format!("**{}**", content));
                } else {
                    parts.push(content);
                }
            } else if tag == "ref" {
                let content = inline(child);
                let target = child.attribute("target").unwrap_or("");
                if target.starts_with("https://") || target.starts_with("http://") {
                    if !content.is_empty() {
                        parts.push(format!("[{}]({})", content, target));
                    } else {
                        parts.push(content);
                    }
                } else {
                    parts.push(content);
                }
            } else if tag == "lb" {
                parts.push("  \n".to_string());
            } else {
                parts.push(inline(child));
            }
            if let Some(tail) = child.tail() {
                parts.push(tail.to_string());
            }
        } else if child.is_text() {
            parts.push(child.text().unwrap_or("").to_string());
        }
    }
    // If element has no children but has direct text (rare case), try text()
    if parts.is_empty() {
        if let Some(text) = element.text() {
            return collapse_spaces(text).trim().to_string();
        }
    }
    let joined = parts.join("");
    collapse_spaces(&joined).trim().to_string()
}
