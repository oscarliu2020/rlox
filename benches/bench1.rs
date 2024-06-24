use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rlox::runner::run_file;
fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("benches");
    group
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(40));
    group.bench_function("bench1", |b| {
        b.iter(|| run_file(black_box("test_data/benches/fib.lox")))
    });
    group.finish();
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
