use std::sync::Arc;

use dashmap::DashSet;
use rspack_core::Plugin;

use crate::common::{compile, compile_fixture_with_plugins, preclude::*};

#[derive(Debug)]
struct NamedChunkTester {
  chunk_ids: Arc<DashSet<String>>,
}

impl Plugin for NamedChunkTester {
  fn name(&self) -> &'static str {
    "NamedChunkTester"
  }

  fn tap_generated_chunk(
    &self,
    _ctx: &rspack_core::BundleContext,
    chunk: &rspack_core::Chunk,
    _bundle_options: &rspack_core::NormalizedBundleOptions,
  ) {
    self.chunk_ids.insert(chunk.id.clone());
  }
}

#[tokio::test]
async fn named_chunk() {
  let chunk_ids: Arc<DashSet<String>> = Default::default();
  compile_fixture_with_plugins(
    "named-chunk",
    vec![Box::new(NamedChunkTester {
      chunk_ids: chunk_ids.clone(),
    })],
  )
  .await;
  assert!(chunk_ids.contains("main"));
  // FIXME: should not contains 'fixtures_named-chunk'
  assert!(chunk_ids.contains("fixtures_named-chunk_src_bar_json"));
  assert!(chunk_ids.contains("fixtures_named-chunk_src_foo_js"));
}
