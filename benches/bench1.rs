use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rlox::runner::run_file;
fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("benches");
    group
        // .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(200))
        .warm_up_time(std::time::Duration::from_secs(10));
    group.bench_function("fib", |b| {
        b.iter(|| run_file(black_box("test_data/benches/fib.lox")))
    });
    group.bench_function("binary_trees", |b| {
        b.iter(|| run_file(black_box("test_data/benches/binary_trees.lox")))
    });
    group.bench_function("equality", |b| {
        b.iter(|| run_file(black_box("test_data/benches/equality.lox")))
    });
    group.bench_function("zoo_batch", |b| {
        b.iter(|| run_file(black_box("test_data/benches/zoo_batch.lox")))
    });
    group.finish();
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
