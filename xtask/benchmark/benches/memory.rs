#![allow(dead_code)]
#![allow(clippy::unwrap_used)]

use criterion::{Criterion, criterion_group, criterion_main};

#[path = "../cases/mod.rs"]
mod cases;
mod groups;

fn configure_rayon_for_benchmark(_: &mut Criterion) {
  rspack_benchmark::configure_rayon_for_benchmark();
}

criterion_group!(benchmark_setup, configure_rayon_for_benchmark);

criterion_main!(
  benchmark_setup,
  cases::build_module_graph::case,
  cases::bundle_css_development::case,
);
