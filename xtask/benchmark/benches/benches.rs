#![allow(clippy::unwrap_used)]

use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};

mod groups;
// Keep these registered case entrypoints in a dedicated short source path:
// CodSpeed embeds file!() into callgrind profile-part names shown by KCachegrind.
#[path = "../cases/mod.rs"]
mod cases;
// Keep these registered stage entrypoints in a dedicated short source path:
// CodSpeed embeds file!() into callgrind profile-part names shown by KCachegrind.
#[path = "../stages/mod.rs"]
mod stages;

fn configure_rayon_for_benchmark(_: &mut Criterion) {
  rspack_benchmark::configure_rayon_for_benchmark();
}

criterion_group! {
  name = benchmark_setup;
  config = Criterion::default()
    .sample_size(10)
    .warm_up_time(Duration::from_millis(1))
    .measurement_time(Duration::from_millis(1));
  targets = configure_rayon_for_benchmark
}

criterion_main!(
  benchmark_setup,
  cases::bundle_misc_pure_functions_off_production_sourcemap::case,
  cases::bundle_misc_pure_functions_on_production_sourcemap::case
);
