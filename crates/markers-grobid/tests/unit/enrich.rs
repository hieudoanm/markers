//! Unit tests for Crossref enrichment parsing and metadata merging.

use markers_grobid::enrich::{Enrichment, apply, parse_message, strip_jats};
use markers_grobid::metadata::Metadata;
use serde_json::json;

#[test]
fn parses_crossref_message() {
    let body = json!({
        "message": {
            "DOI": "10.1/x",
            "type": "journal-article",
            "is-referenced-by-count": 123,
            "title": ["A Paper"],
            "container-title": ["Nature"],
            "published-print": {"date-parts": [[2020, 5, 1]]},
            "abstract": "<jats:p>First &amp; foremost.</jats:p><jats:p>Second.</jats:p>"
        }
    });
    let e = parse_message(&body).unwrap();
    assert_eq!(e.kind.as_deref(), Some("journal-article"));
    assert_eq!(e.cited_by, Some(123));
    assert_eq!(e.year.as_deref(), Some("2020"));
    assert_eq!(e.journal.as_deref(), Some("Nature"));
    assert_eq!(
        e.abstract_text.as_deref(),
        Some("First & foremost. Second.")
    );
}

#[test]
fn parse_message_handles_minimal_payload() {
    let body = json!({"message": {"DOI": "10.1/y"}});
    let e = parse_message(&body).unwrap();
    assert!(e.kind.is_none());
    assert!(e.abstract_text.is_none());
}

#[test]
fn apply_only_fills_empty_fields() {
    let mut meta = Metadata {
        year: Some("1999".into()),
        ..Default::default()
    };
    let e = Enrichment {
        kind: Some("proceedings-article".into()),
        year: Some("2020".into()),
        ..Default::default()
    };
    apply(&mut meta, &e);
    assert_eq!(meta.kind.as_deref(), Some("proceedings-article"));
    assert_eq!(meta.year.as_deref(), Some("1999"));
}

#[test]
fn strip_jats_removes_markup() {
    assert_eq!(
        strip_jats("<jats:p>Hello <italic>world</italic></jats:p>"),
        Some("Hello world".into())
    );
    assert_eq!(strip_jats("<jats:p>   </jats:p>"), None);
}
