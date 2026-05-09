use criterion::criterion_group;
use rspack_benchmark::Criterion;

pub fn bench(c: &mut Criterion) {
  crate::groups::persistent_cache::persistent_cache_restore_benchmark(c);
}

criterion_group!(case, bench);
