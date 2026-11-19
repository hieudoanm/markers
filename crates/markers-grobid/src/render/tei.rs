//! TEI tree helpers: namespace-agnostic local-name queries and text
//! utilities shared by metadata extraction, inline rendering, and
//! bibliography parsing.

use roxmltree::Node;

use crate::utils::text_content;

/// Find the TEI root element from a parsed document's root node.
pub fn tei_root<'a>(root: Node<'a, 'a>) -> Node<'a, 'a> {
    if root.is_element() && root.tag_name().name() == "TEI" {
        return root;
    }
    root.children()
        .find(|n| n.is_element() && n.tag_name().name() == "TEI")
        .unwrap_or(root)
}

/// First descendant element (depth-first) with the given local name.
pub fn descendant_named<'a>(node: Node<'a, 'a>, name: &str) -> Option<Node<'a, 'a>> {
    for child in node.children() {
        if !child.is_element() {
            continue;
        }
        if child.tag_name().name() == name {
            return Some(child);
        }
        if let Some(found) = descendant_named(child, name) {
            return Some(found);
        }
    }
    None
}

/// All descendant elements with the given local name (depth-first).
pub fn descendants_named<'a>(node: Node<'a, 'a>, name: &str) -> Vec<Node<'a, 'a>> {
    let mut out = Vec::new();
    for child in node.children() {
        if !child.is_element() {
            continue;
        }
        if child.tag_name().name() == name {
            out.push(child);
        }
        out.extend(descendants_named(child, name));
    }
    out
}

/// Collapse runs of whitespace (except newlines) into single spaces.
pub(crate) fn collapse_spaces(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() && ch != '\n' {
            if !prev_space {
                result.push(' ');
                prev_space = true;
            }
        } else {
            result.push(ch);
            prev_space = ch == ' ';
        }
    }
    result
}

/// Format an author element's forename/surname parts into a display name.
pub fn author_name(author: Node) -> String {
    let mut parts: Vec<String> = Vec::new();
    for n in author.descendants() {
        if n.is_element() && (n.tag_name().name() == "forename" || n.tag_name().name() == "surname")
        {
            if let Some(t) = n.text() {
                parts.push(t.to_string());
            }
        }
    }
    let name = collapse_spaces(&parts.join(" ")).trim().to_string();
    if name.is_empty() {
        // Corporate authors, <name> elements, etc.
        return text_content(&author);
    }
    name
}
