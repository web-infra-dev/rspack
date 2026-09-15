use criterion::criterion_group;
use rspack_benchmark::Criterion;

pub fn bench(c: &mut Criterion) {
  super::run(
    c,
    crate::groups::compilation_stages::flag_dependency_usage_benchmark,
  );
}

criterion_group!(stage, bench);
