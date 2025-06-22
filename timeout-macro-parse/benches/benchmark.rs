use criterion::{criterion_group, criterion_main, Criterion};
use proc_macro2::TokenStream;
use std::hint::black_box;
use std::str::FromStr;

const SMALL_RAW_FN: &str = r#"
async fn add(first: i64, second: i64) -> i64 {
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    first + second
}
"#;

fn parse_raw_fn(attr: &str) -> TokenStream {
    let attr = TokenStream::from_str(attr).unwrap();
    let input = TokenStream::from_str(SMALL_RAW_FN).unwrap();
    timeout_macro_parse::tokio_timeout(attr, input)
}

const SHORT_DUR_ATTR_WITH_PANIC: &str = r#"duration = "1ms", on_error="panic""#;
const LONG_DUR_ATTR_WITH_PANIC: &str = r#"duration = "1h100ms20m10s25ms15s", on_error="panic""#;

const SHORT_DUR_ATTR_WITH_LONG_PATH: &str =
    r#"duration = "1ms", on_error=path::to::function::with::long::path"#;
const LONG_DUR_ATTR_WITH_LONG_PATH: &str =
    r#"duration = "1h100ms20m10s25ms15s", on_error=path::to::function::with::long::path"#;

const SHORT_DUR_ATTR_WITH_SHORT_PATH: &str = r#"duration = "1ms", on_error=thing"#;
const LONG_DUR_ATTR_WITH_SHORT_PATH: &str = r#"duration = "1h100ms20m10s25ms15s", on_error=thing"#;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("small attr panic", |b| {
        b.iter(|| parse_raw_fn(black_box(SHORT_DUR_ATTR_WITH_PANIC)))
    });
    c.bench_function("long attr panic", |b| {
        b.iter(|| parse_raw_fn(black_box(LONG_DUR_ATTR_WITH_PANIC)))
    });
    c.bench_function("short attr long path", |b| {
        b.iter(|| parse_raw_fn(black_box(SHORT_DUR_ATTR_WITH_LONG_PATH)))
    });
    c.bench_function("long attr long path", |b| {
        b.iter(|| parse_raw_fn(black_box(LONG_DUR_ATTR_WITH_LONG_PATH)))
    });
    c.bench_function("short attr short path", |b| {
        b.iter(|| parse_raw_fn(black_box(SHORT_DUR_ATTR_WITH_SHORT_PATH)))
    });
    c.bench_function("long attr short path", |b| {
        b.iter(|| parse_raw_fn(black_box(LONG_DUR_ATTR_WITH_SHORT_PATH)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
