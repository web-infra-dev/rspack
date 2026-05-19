use criterion::criterion_group;
use rspack_benchmark::Criterion;

pub fn bench(c: &mut Criterion) {
  super::run(
    c,
    crate::groups::compilation_stages::runtime_requirements_benchmark,
  );
}

criterion_group!(stage, bench);
