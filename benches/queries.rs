use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use knowledge_rs::graph::KnowledgeGraph;
use knowledge_rs::query::{CentralityMetric, ConnectedFilesQuery, HubsQuery, Query, ShortestPathQuery};
use knowledge_rs::utils::cache::CacheMode;
use std::path::Path;

fn build_graph_once() -> KnowledgeGraph {
    let root = Path::new(".");
    KnowledgeGraph::build_from_directory_with_cache_opts(root, CacheMode::Use, false)
        .expect("build graph")
}

fn bench_queries(c: &mut Criterion) {
    // Setup outside of iter
    let graph = build_graph_once();

    let mut group = c.benchmark_group("queries");

    // Heuristic pick: pick two files from the repo for path & connected benchmarks
    let mut files: Vec<_> = graph.files.keys().cloned().collect();
    files.sort();
    let from = files.get(0).cloned();
    let to = files.get(1).cloned();

    // Connected files
    if let Some(sample) = from.clone() {
        group.bench_function(BenchmarkId::new("connected_files", sample.file_name().and_then(|s| s.to_str()).unwrap_or("sample")), |b| {
            b.iter(|| {
                let q = ConnectedFilesQuery { file: sample.clone() };
                let res = q.run(black_box(&graph));
                black_box(res.len())
            })
        });
    }

    // Hubs (top 10 by total degree)
    group.bench_function(BenchmarkId::new("hubs", "top10_total"), |b| {
        b.iter(|| {
            let q = HubsQuery { metric: CentralityMetric::Total, top: 10 };
            let res = q.run(black_box(&graph));
            black_box(res.len())
        })
    });

    // Shortest path between two files (if at least two files)
    if let (Some(a), Some(b)) = (from, to) {
        group.bench_function(BenchmarkId::new("shortest_path", "a_to_b"), |bch| {
            bch.iter(|| {
                let q = ShortestPathQuery { from: a.clone(), to: b.clone() };
                let res = q.run(black_box(&graph));
                black_box(res.len())
            })
        });
    }

    group.finish();
}

criterion_group!(name = benches; config = Criterion::default(); targets = bench_queries);
criterion_main!(benches);
