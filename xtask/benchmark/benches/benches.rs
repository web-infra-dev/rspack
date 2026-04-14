#![allow(clippy::unwrap_used)]

use criterion::criterion_main;
use groups::{
  build_chunk_graph::chunk_graph, bundle::bundle, compilation_stages::compilation_stages,
  module_graph_api::module_graph_api, persistent_cache::persistent_cache,
  scan_dependencies::scan_dependencies,
};

mod groups;

criterion_main!(
  chunk_graph,
  module_graph_api,
  scan_dependencies,
  bundle,
  compilation_stages,
  persistent_cache
);
