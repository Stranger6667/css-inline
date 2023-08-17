use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Criterion};
use css_inline::inline_to;
use std::fs;

#[derive(serde::Deserialize, Debug)]
struct Benchmark {
    name: String,
    html: String,
}

fn inlining(c: &mut Criterion) {
    let benchmarks_str =
        fs::read_to_string("../benchmarks/benchmarks.json").expect("Failed to load benchmarks");
    let benchmarks: Vec<Benchmark> =
        serde_json::from_str(&benchmarks_str).expect("Failed to load benchmarks");
    for benchmark in benchmarks.iter() {
        let html = black_box(&benchmark.html);
        c.bench_function(&benchmark.name, |b| {
            let mut output = Vec::with_capacity(
                (html.len() as f64 * 1.5).min(usize::MAX as f64).round() as usize,
            );
            b.iter(|| {
                inline_to(html, &mut output).expect("Inlining failed");
                output.clear();
            })
        });
    }
}

criterion_group!(benches, inlining);
criterion_main!(benches);
