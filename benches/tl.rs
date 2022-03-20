use criterion::*;

// Better real world benchmarks: https://github.com/y21/rust-html-parser-benchmark

const INPUT: &str = r#"
<!doctype html>
<html>
<head>
    <title>Example Domain</title>

    <meta charset="utf-8" />
    <meta http-equiv="Content-type" content="text/html; charset=utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
</head>

<body>
<div>
    <h1>Example Domain</h1>
    <p>This domain is for use in illustrative examples in documents. You may use this
    domain in literature without prior coordination or asking for permission.</p>
    <p><a href="https://www.iana.org/domains/example">More information...</a></p>
</div>
</body>
</html>
"#;

pub fn criterion_benchmark(cr: &mut Criterion) {
    cr.bench_function("tl", |b| {
        b.iter(|| {
            let _ = tl::parse(black_box(INPUT), tl::ParserOptions::default());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
