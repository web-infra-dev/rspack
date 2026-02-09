use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

pub struct ProcessAssetsPass;

#[async_trait]
impl PassExt for ProcessAssetsPass {
  fn name(&self) -> &'static str {
    "process assets"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let plugin_driver = compilation.plugin_driver.clone();
    compilation.process_assets(plugin_driver).await
  }
}

impl Compilation {
  #[instrument("Compilation:process_assets",target=TRACING_BENCH_TARGET, skip_all)]
  async fn process_assets(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let mut process_asset_artifact = self.process_asset_artifact.steal();
    process_asset_artifact.assets = std::mem::take(&mut self.assets);
    process_asset_artifact.assets_related_in = std::mem::take(&mut self.assets_related_in);
    process_asset_artifact.diagnostics = std::mem::take(&mut self.diagnostics);
    process_asset_artifact.records = self.records.take();
    process_asset_artifact.file_dependencies = std::mem::take(&mut self.file_dependencies);
    process_asset_artifact.context_dependencies = std::mem::take(&mut self.context_dependencies);
    process_asset_artifact.code_generated_modules = std::mem::take(&mut self.code_generated_modules);
    let mut build_chunk_graph_artifact = std::mem::take(&mut self.build_chunk_graph_artifact);

    let result = plugin_driver
      .compilation_hooks
      .process_assets
      .call(
        self,
        &mut process_asset_artifact,
        &mut build_chunk_graph_artifact,
      )
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.processAssets"));

    self.assets = std::mem::take(&mut process_asset_artifact.assets);
    self.assets_related_in = std::mem::take(&mut process_asset_artifact.assets_related_in);
    self.diagnostics = std::mem::take(&mut process_asset_artifact.diagnostics);
    self.records = process_asset_artifact.records.take();
    self.file_dependencies = std::mem::take(&mut process_asset_artifact.file_dependencies);
    self.context_dependencies = std::mem::take(&mut process_asset_artifact.context_dependencies);
    self.code_generated_modules = std::mem::take(&mut process_asset_artifact.code_generated_modules);
    self.build_chunk_graph_artifact = build_chunk_graph_artifact;

    self.process_asset_artifact = process_asset_artifact.into();
    result
  }
}
