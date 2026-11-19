//! Block-level TEI → Markdown rendering: headings, paragraphs, lists,
//! tables, formulas, and figures.

use roxmltree::Node;

use super::inline::inline;
use crate::utils::text_content;

fn render_list(element: Node, depth: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    for item in element.children() {
        if item.is_element() && item.tag_name().name() == "item" {
            // look for p or direct content
            let mut content = String::new();
            let mut has_p = false;
            for p in item.children() {
                if p.is_element() && p.tag_name().name() == "p" {
                    content.push_str(&inline(p));
                    content.push(' ');
                    has_p = true;
                }
            }
            if !has_p {
                content = inline(item);
            }
            let content = content.trim().to_string();
            lines.push(format!("{}- {}", "  ".repeat(depth), content));
            for nested in item.children() {
                if nested.is_element() && nested.tag_name().name() == "list" {
                    lines.push(render_list(nested, depth + 1));
                }
            }
        }
    }
    lines.join("\n")
}

fn render_table(element: Node) -> String {
    let mut rows: Vec<Vec<String>> = Vec::new();
    for row in element.descendants() {
        if row.is_element() && row.tag_name().name() == "row" {
            let mut cells: Vec<String> = Vec::new();
            for cell in row.children() {
                if cell.is_element() && cell.tag_name().name() == "cell" {
                    cells.push(inline(cell).replace("|", "\\|"));
                }
            }
            if !cells.is_empty() {
                rows.push(cells);
            }
        }
    }
    if rows.is_empty() {
        return text_content(&element);
    }
    let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let rows_padded: Vec<Vec<String>> = rows
        .into_iter()
        .map(|mut r| {
            while r.len() < width {
                r.push(String::new());
            }
            r
        })
        .collect();
    let mut output: Vec<String> = Vec::new();
    if !rows_padded.is_empty() {
        output.push(format!("| {} |", rows_padded[0].join(" | ")));
        output.push(format!("| {} |", vec!["---"; width].join(" | ")));
        for row in rows_padded[1..].iter() {
            output.push(format!("| {} |", row.join(" | ")));
        }
    }
    output.join("\n")
}

fn render_node(element: Node, depth: usize) -> String {
    let tag = element.tag_name().name();
    if tag == "head" {
        let heading = inline(element);
        let level = if depth < 6 { depth } else { 6 };
        return format!("{} {}", "#".repeat(level), heading);
    }
    if tag == "p" {
        return inline(element);
    }
    if tag == "list" {
        return render_list(element, 0);
    }
    if tag == "table" {
        return render_table(element);
    }
    if tag == "formula" {
        return format!("$$\n{}\n$$", text_content(&element));
    }
    if tag == "figure" {
        let mut label_parts: Vec<String> = Vec::new();
        for l in element.children() {
            if l.is_element() && l.tag_name().name() == "label" {
                label_parts.push(text_content(&l));
            }
        }
        let mut caption_parts: Vec<String> = Vec::new();
        for c in element.children() {
            if c.is_element() && (c.tag_name().name() == "figDesc" || c.tag_name().name() == "head")
            {
                caption_parts.push(text_content(&c));
            }
        }
        let mut graphic_url = String::new();
        for g in element.descendants() {
            if g.is_element() && g.tag_name().name() == "graphic" {
                if let Some(url) = g.attribute("url") {
                    graphic_url = url.to_string();
                    break;
                }
            }
        }
        let description: String = [label_parts.join(" "), caption_parts.join(" ")]
            .into_iter()
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if !graphic_url.is_empty() {
            return format!("![{}]({})", description, graphic_url);
        }
        if !description.is_empty() {
            return format!("> {}", description);
        }
        return String::new();
    }
    if tag == "div" {
        return render_children(element, depth + 1);
    }
    String::new()
}

/// Render an element's block children joined by blank lines.
pub(crate) fn render_children(element: Node, depth: usize) -> String {
    let mut blocks: Vec<String> = Vec::new();
    for child in element.children() {
        if child.is_element() {
            let tag = child.tag_name().name();
            if matches!(
                tag,
                "head" | "p" | "div" | "list" | "table" | "formula" | "figure"
            ) {
                let result = render_node(child, depth);
                if !result.is_empty() {
                    blocks.push(result);
                }
            }
        }
    }
    blocks.join("\n\n")
}
