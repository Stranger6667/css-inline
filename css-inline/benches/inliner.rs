use criterion::{black_box, criterion_group, criterion_main, Criterion};
use css_inline::inline;
use once_cell::sync::Lazy;
use std::fs;

#[derive(serde::Deserialize, Debug)]
struct Benchmark {
    name: String,
    html: String,
}

static BENCHMARKS: Lazy<Vec<Benchmark>> = Lazy::new(|| {
    let benchmarks_str =
        fs::read_to_string("../benchmarks/benchmarks.json").expect("Failed to load benchmarks");
    serde_json::from_str(&benchmarks_str).expect("Failed to load benchmarks")
});

fn inlining(c: &mut Criterion) {
    for benchmark in BENCHMARKS.iter() {
        let html = black_box(&benchmark.html);
        c.bench_function(&benchmark.name, |b| {
            b.iter(|| inline(html).expect("Inlining failed"))
        });
    }
}

criterion_group!(benches, inlining);
criterion_main!(benches);
