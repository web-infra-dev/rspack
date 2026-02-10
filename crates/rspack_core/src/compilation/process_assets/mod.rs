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
    process_asset_artifact.assets = self.assets.clone();
    process_asset_artifact.assets_related_in = self.assets_related_in.clone();
    process_asset_artifact.diagnostics = self.diagnostics.clone();
    process_asset_artifact.records = self.records.take();
    process_asset_artifact.file_dependencies = self.file_dependencies.clone();
    process_asset_artifact.context_dependencies = self.context_dependencies.clone();
    process_asset_artifact.code_generated_modules = self.code_generated_modules.clone();
    let compilation_ptr = self as *const Compilation;
    let build_chunk_graph_artifact_ptr =
      &mut self.build_chunk_graph_artifact as *mut BuildChunkGraphArtifact;
    let result = unsafe {
      plugin_driver
        .compilation_hooks
        .process_assets
        .call(
          &*compilation_ptr,
          &mut process_asset_artifact,
          &mut *build_chunk_graph_artifact_ptr,
        )
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.processAssets"))
    };

    self.assets = std::mem::take(&mut process_asset_artifact.assets);
    self.assets_related_in = std::mem::take(&mut process_asset_artifact.assets_related_in);
    self.diagnostics = std::mem::take(&mut process_asset_artifact.diagnostics);
    self.records = process_asset_artifact.records.take();
    self.file_dependencies = std::mem::take(&mut process_asset_artifact.file_dependencies);
    self.context_dependencies = std::mem::take(&mut process_asset_artifact.context_dependencies);
    self.code_generated_modules =
      std::mem::take(&mut process_asset_artifact.code_generated_modules);

    self.process_asset_artifact = process_asset_artifact.into();
    result
  }
}
