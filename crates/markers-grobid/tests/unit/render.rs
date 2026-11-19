//! Unit tests for TEI → Markdown rendering and reference parsing.

use markers_grobid::render::{
    convert_doc, convert_tei_to_md, convert_tei_xml, extract_references, tei_root,
};

const TEI: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<TEI xmlns="http://www.tei-c.org/ns/1.0">
  <teiHeader>
    <fileDesc>
      <titleStmt><title level="a" type="main">Render Test Title</title></titleStmt>
      <sourceDesc>
        <biblStruct>
          <analytic><author><forename>Ann</forename><surname>Lee</surname></author></analytic>
          <monogr>
            <title level="j">Journal of Rendering</title>
            <imprint>
              <date when="2020">2020</date>
              <biblScope unit="volume">7</biblScope>
              <biblScope unit="page">1-9</biblScope>
            </imprint>
          </monogr>
        </biblStruct>
      </sourceDesc>
    </fileDesc>
    <profileDesc><abstract><p>Abstract text here.</p></abstract></profileDesc>
  </teiHeader>
  <text>
    <body>
      <div>
        <head>Introduction</head>
        <p>First paragraph with <hi rend="italic">emphasis</hi>.</p>
        <list><item><p>Item one</p></item><item><p>Item two</p></item></list>
        <table><row><cell>A</cell><cell>B</cell></row><row><cell>1</cell><cell>2</cell></row></table>
      </div>
    </body>
  </text>
  <back>
    <div type="references">
      <listBibl>
        <biblStruct><monogr><title>Reference One</title></monogr></biblStruct>
        <bibl><title>Reference Two</title></bibl>
      </listBibl>
    </div>
  </back>
</TEI>"#;

#[test]
fn renders_full_document() {
    let md = convert_tei_xml(TEI).unwrap();
    assert!(md.starts_with("---\n"));
    assert!(md.contains("title: \"Render Test Title\""));
    assert!(md.contains("journal: \"Journal of Rendering\""));
    assert!(md.contains("year: 2020"));
    assert!(md.contains("volume: \"7\""));
    assert!(md.contains("pages: \"1-9\""));
    assert!(md.contains("# Render Test Title"));
    assert!(md.contains("**Authors:** Ann Lee"));
    assert!(md.contains("## Abstract"));
    assert!(md.contains("## Article"));
    assert!(md.contains("## Introduction"));
    assert!(md.contains("*emphasis*"));
    assert!(md.contains("- Item one"));
    assert!(md.contains("| A | B |"));
    assert!(md.contains("## References"));
    assert!(md.contains("- Reference One"));
    assert!(md.contains("- Reference Two"));
}

#[test]
fn falls_back_to_filename_title() {
    let xml = TEI.replace(
        "<titleStmt><title level=\"a\" type=\"main\">Render Test Title</title></titleStmt>",
        "<titleStmt/>",
    );
    let doc = roxmltree::Document::parse(&xml).unwrap();
    let md = convert_doc(&doc, None, "fallback-name").unwrap();
    assert!(md.contains("# fallback-name"));
}

#[test]
fn rejects_invalid_xml() {
    assert!(convert_tei_xml("<TEI><unclosed>").is_err());
}

#[test]
fn converts_real_grobid_tei_fixture() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tei/Predicting language outcome after stroke using machine learning - in search of the big data benefit.tei.xml"
    );
    let md = convert_tei_to_md(std::path::Path::new(path), None).unwrap();
    assert!(md.starts_with("---\n"));
    assert!(md.contains("# "));
    assert!(md.contains("## Article"));
    // Real GROBID output includes a reference list.
    assert!(md.contains("## References"));
}

#[test]
fn links_reference_titles_to_doi() {
    let xml = r#"<TEI><text><body/></text><back><div type="references"><listBibl>
      <biblStruct>
        <analytic><author><surname>Doe</surname></author>
          <title>Linked Paper</title></analytic>
        <monogr><title level="j">Nature</title></monogr>
        <idno type="DOI">10.1234/example.1</idno>
      </biblStruct>
      <biblStruct>
        <monogr><title>No DOI Here</title></monogr>
      </biblStruct>
    </listBibl></div></back></TEI>"#;
    let doc = roxmltree::Document::parse(xml).unwrap();
    let tei = tei_root(doc.root());
    let refs = extract_references(tei);
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].doi.as_deref(), Some("10.1234/example.1"));
    assert_eq!(refs[0].title.as_deref(), Some("Linked Paper"));
    assert_eq!(refs[1].doi, None);
    let md = refs[0].render();
    assert!(md.contains("[Linked Paper](https://doi.org/10.1234/example.1)"));
    assert!(refs[1].render().contains("No DOI Here"));
    assert!(!refs[1].render().contains("doi.org"));
}

#[test]
fn extracts_reference_dois_from_real_fixture() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tei/Predicting language outcome after stroke using machine learning - in search of the big data benefit.tei.xml"
    );
    let content = std::fs::read_to_string(path).unwrap();
    let doc = roxmltree::Document::parse(&content).unwrap();
    let refs = extract_references(tei_root(doc.root()));
    assert!(!refs.is_empty());
    let with_doi = refs.iter().filter(|r| r.doi.is_some()).count();
    assert!(with_doi > 0, "expected DOI-bearing references in fixture");
}
