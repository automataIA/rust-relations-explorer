use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_relations_explorer::graph::KnowledgeGraph;
use rust_relations_explorer::utils::cache::CacheMode;
use std::path::Path;

fn bench_build_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_graph");

    // Benchmark different cache modes
    for mode in [CacheMode::Rebuild, CacheMode::Use, CacheMode::Ignore] {
        let label = match mode {
            CacheMode::Rebuild => "rebuild",
            CacheMode::Use => "use_cache",
            CacheMode::Ignore => "ignore_cache",
        };
        group.bench_function(
            BenchmarkId::new("build_from_directory_with_cache_opts", label),
            |b| {
                b.iter(|| {
                    let root = Path::new(".");
                    let graph = KnowledgeGraph::build_from_directory_with_cache_opts(
                        black_box(root),
                        mode,
                        false,
                    )
                    .expect("build graph");
                    // prevent optimizer from discarding
                    black_box(graph.files.len())
                })
            },
        );
    }

    group.finish();
}

criterion_group!(name = benches; config = Criterion::default(); targets = bench_build_graph);
criterion_main!(benches);
