use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

pub struct OptimizeCodeGenerationPass;

#[async_trait]
impl PassExt for OptimizeCodeGenerationPass {
  fn name(&self) -> &'static str {
    "optimize code generation"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let mut build_module_graph_artifact = compilation.build_module_graph_artifact.take();
    let mut diagnostics = vec![];
    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .optimize_code_generation
      .call(
        compilation,
        &mut build_module_graph_artifact,
        &mut diagnostics,
      )
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeCodeGeneration"))?;

    compilation
      .build_module_graph_artifact
      .replace(build_module_graph_artifact);
    compilation.extend_diagnostics(diagnostics);

    Ok(())
  }
}
