use rspack_collections::IdentifierSet;
use rspack_core::{
  BoxModule, Compilation, CompilationBuildModule, CompilationId, CompilationOptimizeDependencies,
  CompilerId, ExportsInfoArtifact, FactoryMeta, Plugin, RuntimeSpec, SideEffectsOptimizeArtifact,
  build_module_graph::BuildModuleGraphArtifact, get_entry_runtime,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug, Default)]
pub struct FlagAllModulesAsUsedPlugin {
  explanation: String,
}

impl FlagAllModulesAsUsedPlugin {
  pub fn new(explanation: String) -> Self {
    Self::new_inner(explanation)
  }
}

impl Plugin for FlagAllModulesAsUsedPlugin {
  fn name(&self) -> &'static str {
    "rspack:FlagAllModulesAsUsedPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));

    ctx
      .compilation_hooks
      .build_module
      .tap(build_module::new(self));

    Ok(())
  }
}

#[plugin_hook(CompilationOptimizeDependencies for FlagAllModulesAsUsedPlugin)]
async fn optimize_dependencies(
  &self,
  compilation: &Compilation,
  _side_effects_optimize_artifact: &mut SideEffectsOptimizeArtifact,
  build_module_graph_artifact: &mut BuildModuleGraphArtifact,
  exports_info_artifact: &mut ExportsInfoArtifact,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<bool>> {
  let entries = &compilation.entries;

  let runtime = compilation
    .entries
    .iter()
    .map(|(name, entry_data)| get_entry_runtime(name, &entry_data.options, entries))
    .fold(RuntimeSpec::default(), |mut a, b| {
      a.extend(&b);
      a
    });

  let mg = build_module_graph_artifact.get_module_graph_mut();

  let module_id_list: IdentifierSet = mg.modules_keys().copied().collect();

  for module_id in module_id_list {
    exports_info_artifact
      .get_exports_info_data_mut(&module_id)
      .set_used_in_unknown_way(Some(&runtime));
  }

  Ok(None)
}

#[plugin_hook(CompilationBuildModule for FlagAllModulesAsUsedPlugin)]
async fn build_module(
  &self,
  _compiler_id: CompilerId,
  _compilation_id: CompilationId,
  module: &mut BoxModule,
) -> Result<()> {
  // set all modules have effects. To avoid any module remove by tree shaking.
  // see: https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/FlagAllModulesAsUsedPlugin.js#L43-L47
  module.set_factory_meta(FactoryMeta {
    side_effect_free: Some(false),
  });

  let module_identifier = module.identifier();
  let build_info = module.build_info_mut();
  if build_info.module_concatenation_bailout.is_none() {
    // webpack avoid those modules be concatenated using add a virtual module_graph_connection.
    // see: https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/FlagAllModulesAsUsedPlugin.js#L42
    // Rspack need incremental build, so we should not add virtual connection to module.
    // We can add a bail reason to avoid those modules be concatenated.
    build_info.module_concatenation_bailout = Some(format!(
      "Module {} is referenced by {}",
      module_identifier, &self.explanation
    ));
  }

  Ok(())
}
