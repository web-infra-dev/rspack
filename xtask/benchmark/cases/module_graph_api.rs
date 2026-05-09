use criterion::criterion_group;
use rspack_benchmark::Criterion;

pub fn bench(c: &mut Criterion) {
  crate::groups::module_graph_api::module_graph_api_benchmark(c);
}

criterion_group!(case, bench);
