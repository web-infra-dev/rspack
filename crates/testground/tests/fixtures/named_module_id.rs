use std::sync::Arc;

use anyhow::ensure;
use dashmap::DashSet;

use crate::common::{compile_fixture_with_plugins, prelude::*};

#[derive(Debug)]
struct NamedModuleIdTester {
  module_ids: Arc<DashSet<String>>,
}

impl Plugin for NamedModuleIdTester {
  fn name(&self) -> &'static str {
    "NamedModuleIdTester"
  }

  fn tap_generated_chunk(
    &self,
    _ctx: &rspack_core::BundleContext,
    chunk: &rspack_core::Chunk,
    _bundle_options: &rspack_core::NormalizedBundleOptions,
  ) {
    self.module_ids.insert(chunk.id.clone());
  }
}
#[ignore = "TODO: Support module_parsed hook"]
#[tokio::test]
async fn named_module_id() -> anyhow::Result<()> {
  let module_ids: Arc<DashSet<String>> = Default::default();
  compile_fixture_with_plugins(
    "named-module-id",
    vec![Box::new(NamedModuleIdTester {
      module_ids: module_ids.clone(),
    })],
  )
  .await;
  ensure!(module_ids.contains("main"));
  ensure!(module_ids.contains("src_bar_json"));
  ensure!(module_ids.contains("src_foo_js"));
  Ok(())
}
