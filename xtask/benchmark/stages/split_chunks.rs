use criterion::criterion_group;
use rspack_benchmark::Criterion;

pub fn bench(c: &mut Criterion) {
  super::run(c, crate::groups::compilation_stages::split_chunks_benchmark);
  super::run(
    c,
    crate::groups::compilation_stages::split_chunks_optimize_for_size_benchmark,
  );
}

criterion_group!(stage, bench);
