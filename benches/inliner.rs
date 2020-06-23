use criterion::{black_box, criterion_group, criterion_main, Criterion};
use css_inline::inline;

fn simple(c: &mut Criterion) {
    let html = black_box(
        r#"<html>
<head>
    <title>Test</title>
    <style>
        h1, h2 { color:blue; }
        strong { text-decoration:none }
        p { font-size:2px }
        p.footer { font-size: 1px}
    </style>
</head>
<body>
    <h1>Big Text</h1>
    <p>
        <strong>Solid</strong>
    </p>
    <p class="footer">Foot notes</p>
</body>
</html>"#,
    );
    c.bench_function("simple HTML", |b| b.iter(|| inline(html).unwrap()));
}

fn merging(c: &mut Criterion) {
    let html = black_box(
        r#"<html>
<head>
    <title>Test</title>
    <style>
        h1, h2 { color:blue; }
        strong { text-decoration:none }
        p { font-size:2px }
        p.footer { font-size: 1px}
    </style>
</head>
<body>
    <h1 style="background-color: black;">Big Text</h1>
    <p style="background-color: black;">
        <strong style="background-color: black;">Solid</strong>
    </p>
    <p class="footer" style="background-color: black;">Foot notes</p>
</body>
</html>"#,
    );
    c.bench_function("merging styles", |b| b.iter(|| inline(html).unwrap()));
}

criterion_group!(benches, simple, merging);
criterion_main!(benches);
