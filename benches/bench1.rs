use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rlox::runner::run_file;
fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("bench1", |b| b.iter(|| run_file(black_box("benches/bench1.rs"))));
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
