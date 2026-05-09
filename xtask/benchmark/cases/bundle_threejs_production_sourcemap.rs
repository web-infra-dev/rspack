use criterion::criterion_group;
use rspack_benchmark::Criterion;

pub fn bench(c: &mut Criterion) {
  crate::groups::bundle::bundle_benchmark_case(c, "threejs-production-sourcemap");
}

criterion_group!(case, bench);
