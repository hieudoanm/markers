//! Citation graph data and dashboard HTML rendering.

use serde::Serialize;

use super::{CorpusDb, Stats};

#[derive(Serialize)]
pub struct GraphNode {
    pub id: String,
    pub doi: Option<String>,
    pub title: String,
    pub authors: String,
    pub year: String,
    pub journal: String,
    pub kind: String,
    pub cited_by: Option<i64>,
    pub external: bool,
    pub degree: i64,
}

#[derive(Serialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
}

impl CorpusDb {
    /// Nodes and edges for the citation graph: corpus works plus external
    /// DOI-only stub nodes for referenced works not in the corpus.
    pub fn graph_data(&self) -> anyhow::Result<(Vec<GraphNode>, Vec<GraphEdge>)> {
        let mut nodes: Vec<GraphNode> = Vec::new();
        // Untitled stub works are excluded from the graph (doi app parity);
        // they still count in corpus stats and search.
        let mut stmt = self.conn.prepare(
            "SELECT key, doi, title, authors, year, journal, kind, cited_by
             FROM works WHERE title != '' ORDER BY key",
        )?;
        let works = stmt
            .query_map([], |r| {
                Ok(GraphNode {
                    id: r.get::<_, String>(0)?,
                    doi: r.get(1)?,
                    title: r.get::<_, String>(2)?,
                    authors: r.get::<_, String>(3)?,
                    year: r.get::<_, String>(4)?,
                    journal: r.get::<_, String>(5)?,
                    kind: r.get::<_, String>(6)?,
                    cited_by: r.get(7)?,
                    external: false,
                    degree: 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        nodes.extend(works);

        // External nodes: referenced DOIs that are not corpus works.
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT r.ref_doi FROM refs r
             LEFT JOIN works w ON w.doi = r.ref_doi
             WHERE r.ref_doi IS NOT NULL AND r.ref_doi != '' AND w.id IS NULL",
        )?;
        let external_dois: Vec<String> = stmt
            .query_map([], |r| r.get(0))?
            .collect::<Result<_, _>>()?;
        for doi in external_dois {
            nodes.push(GraphNode {
                id: format!("ext:{doi}"),
                doi: Some(doi.clone()),
                title: String::new(),
                authors: String::new(),
                year: String::new(),
                journal: String::new(),
                kind: String::new(),
                cited_by: None,
                external: true,
                degree: 0,
            });
        }

        // Edges: work -> referenced DOI. Self-loops and stub refs skipped.
        let mut edges: Vec<GraphEdge> = Vec::new();
        let mut stmt = self.conn.prepare(
            "SELECT work_key, ref_doi FROM refs
             WHERE ref_doi IS NOT NULL AND ref_doi != '' AND is_self = 0",
        )?;
        let raw_edges: Vec<(String, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<_, _>>()?;
        let node_ids: std::collections::HashSet<&str> =
            nodes.iter().map(|n| n.id.as_str()).collect();
        let work_doi: std::collections::HashMap<String, String> = {
            let mut stmt = self
                .conn
                .prepare("SELECT key, doi FROM works WHERE doi IS NOT NULL")?;
            let rows =
                stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
            rows.collect::<Result<_, _>>()?
        };
        for (work_key, ref_doi) in raw_edges {
            // Edge source is the work's DOI node id when it has one, else key.
            let source = work_doi
                .get(&work_key)
                .cloned()
                .unwrap_or_else(|| work_key.clone());
            let target = format!("ext:{ref_doi}");
            let target = if node_ids.contains(target.as_str()) {
                target
            } else {
                ref_doi.clone()
            };
            // Drop edges from stub-excluded works or to missing nodes.
            if source != target && node_ids.contains(source.as_str()) {
                edges.push(GraphEdge { source, target });
            }
        }

        // In-degree per node for radius scaling.
        let mut degree: std::collections::HashMap<String, i64> = Default::default();
        for e in &edges {
            *degree.entry(e.target.clone()).or_insert(0) += 1;
        }
        for n in &mut nodes {
            n.degree = *degree.get(&n.id).unwrap_or(&0);
        }

        Ok((nodes, edges))
    }
}

const GRAPH_TEMPLATE: &str = include_str!("../templates/graph.html");

fn stats_json(stats: &Stats) -> serde_json::Value {
    serde_json::json!({
        "works": stats.works,
        "titled": stats.titled,
        "references": stats.references,
        "doi_refs": stats.doi_refs,
        "stub_refs": stats.stub_refs,
        "self_refs": stats.self_refs,
        "distinct_authors": stats.distinct_authors,
        "min_year": stats.min_year,
        "max_year": stats.max_year,
        "years": stats.year_hist,
        "top_authors": stats.top_authors,
        "top_in_cited": stats.top_in_cited,
        "top_citing": stats.top_citing,
        "top_cited_crossref": stats.top_cited,
    })
}

/// Render the corpus as a self-contained Obsidian-style dashboard page:
/// citation graph plus corpus stats, year chart, and unified search filter.
pub fn render_graph_html(db: &CorpusDb) -> anyhow::Result<String> {
    let (nodes, edges) = db.graph_data()?;
    let stats = db.stats()?;
    let nodes_json = serde_json::to_string(&nodes)?;
    let edges_json = serde_json::to_string(&edges)?;
    Ok(GRAPH_TEMPLATE
        .replace("__NODES__", &nodes_json)
        .replace("__EDGES__", &edges_json)
        .replace("__STATS__", &stats_json(&stats).to_string())
        .replace("__TITLE__", "markers grobid corpus"))
}
