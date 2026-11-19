//! Unit tests for TEI header metadata extraction and frontmatter output.

use markers_grobid::metadata::{Metadata, extract_metadata_doc, frontmatter};

const TEI: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<TEI xmlns="http://www.tei-c.org/ns/1.0">
  <teiHeader>
    <fileDesc>
      <titleStmt>
        <title level="a" type="main">A Study of Things</title>
        <title level="a" type="sub">A Study of Things: Supplement</title>
      </titleStmt>
      <publicationStmt><publisher>Test Publisher</publisher></publicationStmt>
      <sourceDesc>
        <biblStruct>
          <analytic>
            <author><forename>Ada</forename><surname>Lovelace</surname></author>
            <author><forename>Alan</forename><surname>Turing</surname></author>
          </analytic>
          <monogr>
            <title level="j">Journal of Tests</title>
            <imprint>
              <date when="2021-05-01">May 2021</date>
              <biblScope unit="volume">12</biblScope>
              <biblScope unit="issue">3</biblScope>
              <biblScope unit="page">45-67</biblScope>
            </imprint>
          </monogr>
        </biblStruct>
      </sourceDesc>
    </fileDesc>
    <profileDesc>
      <abstract><p>Short abstract.</p></abstract>
    </profileDesc>
    <idno type="DOI">10.1234/test.2021</idno>
  </teiHeader>
  <text>
    <body>
      <div><head>Introduction</head><p>Hello world.</p></div>
    </body>
  </text>
  <back>
    <div type="references">
      <listBibl>
        <biblStruct>
          <analytic><author><forename>Ref</forename><surname>Author</surname></author></analytic>
          <monogr><title>Some Reference</title></monogr>
        </biblStruct>
      </listBibl>
    </div>
  </back>
</TEI>"#;

#[test]
fn extracts_all_metadata_fields() {
    let doc = roxmltree::Document::parse(TEI).unwrap();
    let meta = extract_metadata_doc(&doc);
    assert_eq!(meta.title.as_deref(), Some("A Study of Things"));
    assert_eq!(meta.authors, vec!["Ada Lovelace", "Alan Turing"]);
    assert_eq!(meta.journal.as_deref(), Some("Journal of Tests"));
    assert_eq!(meta.year.as_deref(), Some("2021"));
    assert_eq!(meta.volume.as_deref(), Some("12"));
    assert_eq!(meta.issue.as_deref(), Some("3"));
    assert_eq!(meta.pages.as_deref(), Some("45-67"));
    assert_eq!(meta.doi.as_deref(), Some("10.1234/test.2021"));
    assert_eq!(
        meta.doi_link.as_deref(),
        Some("https://doi.org/10.1234/test.2021")
    );
}

#[test]
fn does_not_pick_up_reference_authors() {
    let doc = roxmltree::Document::parse(TEI).unwrap();
    let meta = extract_metadata_doc(&doc);
    assert!(!meta.authors.iter().any(|a| a.contains("Ref")));
}

#[test]
fn handles_missing_header() {
    let doc = roxmltree::Document::parse("<TEI/>").unwrap();
    let meta = extract_metadata_doc(&doc);
    assert!(meta.title.is_none());
    assert!(meta.authors.is_empty());
}

#[test]
fn frontmatter_contains_fields() {
    let meta = Metadata {
        title: Some("T".into()),
        authors: vec!["A B".into()],
        year: Some("2020".into()),
        doi: Some("10.1/x".into()),
        ..Default::default()
    };
    let fm = frontmatter(&meta);
    assert!(fm.starts_with("---\n"));
    assert!(fm.contains("title: \"T\""));
    assert!(fm.contains("authors: \"A B\""));
    assert!(fm.contains("year: 2020"));
    assert!(fm.contains("doi: \"10.1/x\""));
}
