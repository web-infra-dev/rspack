#![allow(clippy::unwrap_used)]

use criterion::{Criterion, criterion_group, criterion_main};
use groups::{
  build_chunk_graph::chunk_graph, bundle::bundle, compilation_stages::compilation_stages,
  module_graph_api::module_graph_api, persistent_cache::persistent_cache,
  scan_dependencies::scan_dependencies,
};
use rspack_core::configure_rayon_current_thread_for_codspeed;

mod groups;

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
  compilation_stages,
  persistent_cache
);
