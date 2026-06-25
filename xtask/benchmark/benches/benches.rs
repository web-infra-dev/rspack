#![allow(clippy::unwrap_used)]

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

criterion_group!(benchmark_setup, configure_rayon_for_benchmark);

criterion_main!(
  benchmark_setup,
  cases::build_chunk_graph::case,
  cases::build_module_graph::case,
  cases::module_graph_api::case,
  cases::scan_dependencies::case,
  cases::bundle_basic_react_development::case,
  cases::bundle_basic_react_production_sourcemap::case,
  cases::bundle_threejs_development::case,
  cases::bundle_threejs_production_sourcemap::case,
  stages::flag_dependency_exports::stage,
  stages::flag_dependency_usage::stage,
  stages::create_module_ids::stage,
  stages::create_named_module_ids::stage,
  stages::split_chunks::stage,
  stages::create_chunk_ids::stage,
  stages::create_named_chunk_ids::stage,
  stages::mangle_exports::stage,
  stages::create_module_hashes::stage,
  stages::runtime_requirements::stage,
  stages::create_chunk_hashes::stage,
  stages::create_full_hash::stage,
  stages::create_module_assets::stage,
  stages::create_chunk_assets::stage,
  stages::real_content_hash::stage,
  stages::create_concatenate_module::stage,
  stages::concatenate_module_code_generation::stage,
  cases::persistent_cache_restore::case,
  cases::persistent_cache_restore_after_single_file_change::case
);
