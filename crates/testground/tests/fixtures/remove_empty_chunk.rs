use std::sync::Arc;

use anyhow::ensure;
use dashmap::DashSet;
use rspack_core::PluginTapGeneratedChunkHookOutput;

use crate::common::{compile_fixture_with_plugins, prelude::*};

#[derive(Debug)]
struct Tester {
  chunk_ids: Arc<DashSet<String>>,
}

impl Plugin for Tester {
  fn name(&self) -> &'static str {
    "NamedChunkTester"
  }

  fn tap_generated_chunk(
    &self,
    _ctx: &rspack_core::BundleContext,
    chunk: &rspack_core::Chunk,
    _bundle_options: &rspack_core::NormalizedBundleOptions,
  ) -> PluginTapGeneratedChunkHookOutput {
    self.chunk_ids.insert(chunk.id.clone());
    Ok(())
  }
}

#[tokio::test]
async fn remove_empty_chunk() -> anyhow::Result<()> {
  let chunk_ids: Arc<DashSet<String>> = Default::default();
  compile_fixture_with_plugins(
    "remove-empty-chunk",
    vec![Box::new(Tester {
      chunk_ids: chunk_ids.clone(),
    })],
  )
  .await;
  ensure!(chunk_ids.contains("main"));
  ensure!(!chunk_ids.contains("src_foo_js"));
  Ok(())
}
