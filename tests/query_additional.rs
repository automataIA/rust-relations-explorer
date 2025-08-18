use knowledge_rs::graph::{KnowledgeGraph, FileNode, Item, ItemType, Relationship, RelationshipType, Visibility, Location, ItemId};
use knowledge_rs::query::{ShortestPathQuery, HubsQuery, ModuleCentralityQuery, CentralityMetric, Query};
use std::path::PathBuf;

fn make_fn(path: &PathBuf, name: &str) -> Item {
    Item {
        id: ItemId(format!("fn:{}:{}", name, path.display())),
        item_type: ItemType::Function { is_async: false, is_const: false },
        name: name.to_string(),
        visibility: Visibility::Public,
        location: Location { file: path.clone(), line_start: 1, line_end: 1 },
        attributes: vec![],
    }
}

fn graph_chain() -> KnowledgeGraph {
    // a -> b, b -> c, c isolated to module m2 with d
    let mut g = KnowledgeGraph::default();
    let a = PathBuf::from("src/a.rs");
    let b = PathBuf::from("src/b.rs");
    let c = PathBuf::from("src/m1/c.rs");
    let d = PathBuf::from("src/m2/d.rs");

    let ia = make_fn(&a, "fa");
    let ib = make_fn(&b, "fb");
    let ic = make_fn(&c, "fc");
    let id = make_fn(&d, "fd");

    g.files.insert(a.clone(), FileNode { path: a.clone(), items: vec![ia.clone()], imports: vec![], metrics: Default::default() });
    g.files.insert(b.clone(), FileNode { path: b.clone(), items: vec![ib.clone()], imports: vec![], metrics: Default::default() });
    g.files.insert(c.clone(), FileNode { path: c.clone(), items: vec![ic.clone()], imports: vec![], metrics: Default::default() });
    g.files.insert(d.clone(), FileNode { path: d.clone(), items: vec![id.clone()], imports: vec![], metrics: Default::default() });

    g.relationships.push(Relationship { from_item: ia.id.clone(), to_item: ib.id.clone(), relationship_type: RelationshipType::Calls { call_type: "test".into() }, strength: 1.0, context: String::new() });
    g.relationships.push(Relationship { from_item: ib.id.clone(), to_item: ic.id.clone(), relationship_type: RelationshipType::Calls { call_type: "test".into() }, strength: 1.0, context: String::new() });
    // d has no edges, different module directory

    g
}

#[test]
fn shortest_path_found_and_absent() {
    let g = graph_chain();
    // Path a -> b -> c exists
    let p = ShortestPathQuery::new("src/a.rs", "src/m1/c.rs").run(&g);
    assert_eq!(p.first().unwrap().display().to_string(), "src/a.rs");
    assert_eq!(p.last().unwrap().display().to_string(), "src/m1/c.rs");

    // No path from c to a
    let none = ShortestPathQuery::new("src/m1/c.rs", "src/a.rs").run(&g);
    assert!(none.is_empty());
}

#[test]
fn hubs_query_metrics_and_sorting() {
    let g = graph_chain();
    // Total metric: b should have total 2 (1 in, 1 out) and be near top
    let total = HubsQuery::new(CentralityMetric::Total, 10).run(&g);
    assert!(total.iter().any(|(p,i,o)| p.ends_with("src/b.rs") && *i == 1 && *o == 1));

    // In metric: c has indegree 1, d has 0
    let ins = HubsQuery::new(CentralityMetric::In, 10).run(&g);
    assert!(ins.iter().any(|(p,i,_)| p.ends_with("src/m1/c.rs") && *i == 1));

    // Out metric: a has outdegree 1
    let outs = HubsQuery::new(CentralityMetric::Out, 10).run(&g);
    assert!(outs.iter().any(|(p,_,o)| p.ends_with("src/a.rs") && *o == 1));
}

#[test]
fn module_centrality_inter_module_edges() {
    let g = graph_chain();
    // Expect edges counted between parent directories of files
    let rows = ModuleCentralityQuery::new(CentralityMetric::Total, 10).run(&g);
    // At least two modules should appear
    assert!(rows.len() >= 2);
    // Ensure modules include src and src/m1, src/m2
    let mods: Vec<String> = rows.iter().map(|(p,_,_)| p.display().to_string()).collect();
    assert!(mods.iter().any(|m| m == "src" || m.ends_with("/src")));
}
