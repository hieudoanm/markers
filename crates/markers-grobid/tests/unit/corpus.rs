//! Unit tests for the SQLite corpus index: upsert, search, stats,
//! graph data, JSON export, and dashboard HTML.

use markers_grobid::corpus::{CorpusDb, Work, open_db, render_graph_html};
use markers_grobid::render::ParsedReference;
use std::path::PathBuf;

fn sample_work() -> Work {
    Work {
        key: "10.1/test".into(),
        doi: Some("10.1/test".into()),
        doi_link: Some("https://doi.org/10.1/test".into()),
        title: "Test Paper".into(),
        authors: vec!["Ada Lovelace".into()],
        year: Some("2024".into()),
        journal: Some("Journal of Tests".into()),
        volume: Some("3".into()),
        issue: Some("2".into()),
        pages: Some("10-20".into()),
        kind: Some("journal-article".into()),
        abstract_text: Some("An abstract.".into()),
        cited_by: Some(42),
        md_file: Some("test.md".into()),
        pdf_file: Some("test.pdf".into()),
        references: vec![
            ParsedReference {
                title: Some("Other Paper".into()),
                doi: Some("10.2/other".into()),
                year: Some("2020".into()),
                ..Default::default()
            },
            ParsedReference {
                title: None,
                doi: None,
                ..Default::default()
            },
        ],
    }
}

fn temp_db() -> (CorpusDb, PathBuf) {
    let path = std::env::temp_dir().join(format!(
        "grobid-test-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let db = open_db(&path).unwrap();
    (db, path)
}

#[test]
fn upsert_and_search_roundtrip() {
    let (mut db, path) = temp_db();
    db.upsert_work(&sample_work()).unwrap();

    let hits = db.search("test paper", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].doi.as_deref(), Some("10.1/test"));

    let hits = db.search("lovelace", 10).unwrap();
    assert_eq!(hits.len(), 1);

    let hits = db.search("nonexistent", 10).unwrap();
    assert!(hits.is_empty());

    // LIKE wildcards in the query are escaped, not treated as patterns.
    let hits = db.search("%", 10).unwrap();
    assert!(hits.is_empty());

    let _ = std::fs::remove_file(&path);
}

#[test]
fn stats_and_graph_data() {
    let (mut db, path) = temp_db();
    db.upsert_work(&sample_work()).unwrap();

    let s = db.stats().unwrap();
    assert_eq!(s.works, 1);
    assert_eq!(s.references, 2);
    assert_eq!(s.doi_refs, 1);
    assert_eq!(s.stub_refs, 1);
    assert_eq!(s.self_refs, 0);
    assert_eq!(s.min_year, "2024");
    assert_eq!(s.top_cited.len(), 1);
    assert_eq!(s.top_cited[0].2, 42);

    let (nodes, edges) = db.graph_data().unwrap();
    // 1 work node + 1 external DOI node.
    assert_eq!(nodes.len(), 2);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].source, "10.1/test");
    assert_eq!(edges[0].target, "ext:10.2/other");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn upsert_replaces_references() {
    let (mut db, path) = temp_db();
    let mut w = sample_work();
    db.upsert_work(&w).unwrap();
    w.references.clear();
    db.upsert_work(&w).unwrap();
    let s = db.stats().unwrap();
    assert_eq!(s.references, 0);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn export_index_writes_json() {
    let (mut db, path) = temp_db();
    db.upsert_work(&sample_work()).unwrap();
    let index = std::env::temp_dir().join(format!("grobid-idx-{}.json", std::process::id()));
    db.export_index(&index).unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&index).unwrap()).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["doi"], "10.1/test");
    let _ = std::fs::remove_file(&index);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn graph_html_contains_data() {
    let (mut db, path) = temp_db();
    db.upsert_work(&sample_work()).unwrap();
    let html = render_graph_html(&db).unwrap();
    assert!(html.contains("d3@7"));
    assert!(html.contains("10.1/test"));
    assert!(html.contains("10.2/other"));
    let _ = std::fs::remove_file(&path);
}
