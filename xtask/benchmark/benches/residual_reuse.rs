//! Local wall-time runner for the existing CodSpeed SplitChunks stage cases.

#![allow(dead_code, clippy::unwrap_used)]

mod groups;

use criterion::{criterion_group, criterion_main};
use rspack_benchmark::Criterion;

fn bench(c: &mut Criterion) {
  rspack_benchmark::configure_rayon_for_benchmark();
  rspack_tasks::within_compiler_context_for_testing_sync(|| {
    let runtime = rspack_benchmark::build_tokio_rt();
    let _guard = runtime.enter();
    groups::compilation_stages::split_chunks_dedup_depth_benchmark(c, &runtime);
  });
}

criterion_group!(stage, bench);
criterion_main!(stage);
