#![allow(clippy::unwrap_used)]

use criterion::{Criterion, criterion_group, criterion_main};

#[path = "groups/rspack_sources.rs"]
mod rspack_sources;

fn configure_rayon_for_benchmark(_: &mut Criterion) {
  rspack_benchmark::configure_rayon_for_benchmark();
}

criterion_group!(benchmark_setup, configure_rayon_for_benchmark);
criterion_group!(
  rspack_sources_benches,
  rspack_sources::rspack_sources_benchmark
);
criterion_main!(benchmark_setup, rspack_sources_benches);
