use criterion::criterion_group;
use rspack_benchmark::Criterion;

pub fn bench(c: &mut Criterion) {
  crate::groups::build_chunk_graph::build_chunk_graph_benchmark(c);
}

criterion_group!(case, bench);
