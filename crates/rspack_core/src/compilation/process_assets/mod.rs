use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

#[derive(Debug, Default)]
pub struct ProcessAssetsArtifact {
  pub assets: CompilationAssets,
  pub diagnostics: Vec<Diagnostic>,
  pub code_generated_modules: IdentifierSet,
  pub file_dependencies: ArcPathIndexSet,
  pub context_dependencies: ArcPathIndexSet,
  pub records: Option<CompilationRecords>,
}

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
    let mut artifact = ProcessAssetsArtifact {
      assets: mem::take(&mut self.assets),
      diagnostics: mem::take(&mut self.diagnostics),
      code_generated_modules: mem::take(&mut self.code_generated_modules),
      file_dependencies: mem::take(&mut self.file_dependencies),
      context_dependencies: mem::take(&mut self.context_dependencies),
      records: mem::take(&mut self.records),
    };
    let mut build_chunk_graph_artifact = mem::take(&mut self.build_chunk_graph_artifact);

    let result = plugin_driver
      .compilation_hooks
      .process_assets
      .call(self, &mut artifact, &mut build_chunk_graph_artifact)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.processAssets"));

    self.assets = artifact.assets;
    self.diagnostics = artifact.diagnostics;
    self.code_generated_modules = artifact.code_generated_modules;
    self.file_dependencies = artifact.file_dependencies;
    self.context_dependencies = artifact.context_dependencies;
    self.records = artifact.records;
    self.build_chunk_graph_artifact = build_chunk_graph_artifact;

    result
  }
}
