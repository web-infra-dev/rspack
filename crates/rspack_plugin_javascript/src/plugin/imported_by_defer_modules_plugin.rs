use rspack_core::{
  Compilation, CompilationOptimizeChunkModules, DependencyType, ImportedByDeferModulesArtifact,
  Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug, Default)]
pub struct ImportedByDeferModulesPlugin;

#[plugin_hook(CompilationOptimizeChunkModules for ImportedByDeferModulesPlugin)]
async fn optimize_chunk_modules(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  if !compilation.options.experiments.defer_import {
    return Ok(None);
  }

  let mut imported_by_defer_modules_artifact = ImportedByDeferModulesArtifact::default();
  let module_graph = compilation.get_module_graph();
  for (_, dep) in module_graph.dependencies() {
    if dep.get_phase().is_defer()
      && matches!(
        dep.dependency_type(),
        DependencyType::EsmImport | DependencyType::EsmExportImport
      )
      && let Some(module) = module_graph.module_identifier_by_dependency_id(dep.id())
    {
      imported_by_defer_modules_artifact.insert(*module);
    }
  }
  compilation.imported_by_defer_modules_artifact = imported_by_defer_modules_artifact.into();

  Ok(None)
}

impl Plugin for ImportedByDeferModulesPlugin {
  fn name(&self) -> &'static str {
    "ImportedByDeferModulesPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .optimize_chunk_modules
      .tap(optimize_chunk_modules::new(self));
    Ok(())
  }
}
