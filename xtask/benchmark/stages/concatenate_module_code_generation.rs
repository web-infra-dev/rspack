use criterion::criterion_group;
use rspack_benchmark::Criterion;

pub fn bench(c: &mut Criterion) {
  super::run(
    c,
    crate::groups::compilation_stages::concatenate_module_code_generation_benchmark,
  );
}

criterion_group!(stage, bench);
