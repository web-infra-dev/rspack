use std::sync::Arc;

use anyhow::ensure;
use dashmap::DashSet;

use crate::common::{compile_fixture_with_plugins, prelude::*};

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
async fn named_chunk() -> anyhow::Result<()> {
  let chunk_ids: Arc<DashSet<String>> = Default::default();
  compile_fixture_with_plugins(
    "named-chunk",
    vec![Box::new(NamedChunkTester {
      chunk_ids: chunk_ids.clone(),
    })],
  )
  .await;
  ensure!(chunk_ids.contains("main"));
  ensure!(chunk_ids.contains("src_bar_json"));
  ensure!(chunk_ids.contains("src_foo_js"));
  Ok(())
}
