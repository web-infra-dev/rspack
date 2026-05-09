use criterion::criterion_group;
use rspack_benchmark::Criterion;

pub fn bench(c: &mut Criterion) {
  crate::groups::scan_dependencies::benchmark_scan_dependencies(c);
}

criterion_group!(case, bench);
