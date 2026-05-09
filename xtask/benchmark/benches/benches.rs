#![allow(clippy::unwrap_used)]

use criterion::{Criterion, criterion_group, criterion_main};
use groups::{
  build_chunk_graph::chunk_graph, bundle::bundle, module_graph_api::module_graph_api,
  persistent_cache::persistent_cache, scan_dependencies::scan_dependencies,
};
use rspack_core::configure_rayon_current_thread_for_codspeed;

mod groups;
// Keep these registered stage entrypoints in a dedicated short source path:
// CodSpeed embeds file!() into callgrind profile-part names shown by KCachegrind.
#[path = "../stages/mod.rs"]
mod stages;

fn configure_rayon_for_codspeed(_: &mut Criterion) {
  configure_rayon_current_thread_for_codspeed();
}

criterion_group!(codspeed_setup, configure_rayon_for_codspeed);

criterion_main!(
  codspeed_setup,
  chunk_graph,
  module_graph_api,
  scan_dependencies,
  bundle,
  stages::flag_dependency_exports::stage,
  stages::flag_dependency_usage::stage,
  stages::create_module_ids::stage,
  stages::split_chunks::stage,
  stages::create_chunk_ids::stage,
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
  persistent_cache
);
