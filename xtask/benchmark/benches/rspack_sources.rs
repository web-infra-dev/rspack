#![allow(clippy::unwrap_used)]

use criterion::{criterion_group, criterion_main};

#[path = "groups/rspack_sources.rs"]
mod rspack_sources;

criterion_group!(
  rspack_sources_benches,
  rspack_sources::rspack_sources_benchmark
);
criterion_main!(rspack_sources_benches);
