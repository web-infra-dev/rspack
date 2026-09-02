use rspack_collections::IdentifierSet;
use rspack_core::{
  BoxModule, Compilation, CompilationOptimizeDependencies, ExportsInfoArtifact, FactoryMeta,
  ModuleFactoryCreateData, NormalModuleCreateData, NormalModuleFactoryModule, Plugin, RuntimeSpec,
  SideEffectsOptimizeArtifact, build_module_graph::BuildModuleGraphArtifact, get_entry_runtime,
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
      .normal_module_factory_hooks
      .module
      .tap(nmf_module::new(self));

    Ok(())
  }
}

// Write the concatenation bailout at a negative stage so it lands before other
// `optimize_dependencies` taps that read it back, such as `EsmLibraryPlugin`,
// which calls `get_concatenation_bailout_reason` from the same hook.
#[plugin_hook(CompilationOptimizeDependencies for FlagAllModulesAsUsedPlugin, stage = -10)]
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

  let module_id_list: IdentifierSet = build_module_graph_artifact
    .get_module_graph_mut()
    .modules_keys()
    .copied()
    .collect();

  for module_id in &module_id_list {
    exports_info_artifact
      .get_exports_info_data_mut(module_id)
      .set_used_in_unknown_way(Some(&runtime));
  }

  // webpack avoids concatenating these modules by adding a virtual
  // module_graph_connection.
  // see: https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/FlagAllModulesAsUsedPlugin.js#L44
  // Rspack needs incremental build, so we should not add a virtual connection to
  // the module. We can add a bail reason to avoid those modules being
  // concatenated.
  let mg = build_module_graph_artifact.get_module_graph_mut();
  for module_id in &module_id_list {
    if let Some(module) = mg.module_by_identifier_mut(module_id) {
      let build_info = module.build_info_mut();
      if build_info.module_concatenation_bailout.is_none() {
        build_info.module_concatenation_bailout = Some(format!(
          "Module {} is referenced by {}",
          module_id, &self.explanation
        ));
      }
    }
  }

  Ok(None)
}

// Set all modules as having side effects, so tree shaking keeps them.
//
// This runs in the factory phase rather than in `optimizeDependencies`, matching
// webpack: lazy barrel classification reads `factory_meta` while the module
// graph is still being built, which is long before the seal phase.
// see: https://github.com/webpack/webpack/blob/ce97d583e1cd8f3e47b70737de72e91b567a8497/lib/dll/DllPlugin.js#L72-L88
//
// The stage keeps this after `SideEffectsFlagPlugin`, whose tap runs at the
// default stage and would otherwise overwrite this value.
#[plugin_hook(NormalModuleFactoryModule for FlagAllModulesAsUsedPlugin, stage = 10, tracing=false)]
async fn nmf_module(
  &self,
  _data: &mut ModuleFactoryCreateData,
  _create_data: &NormalModuleCreateData,
  module: &mut BoxModule,
) -> Result<()> {
  module.set_factory_meta(FactoryMeta {
    side_effect_free: Some(false),
  });

  Ok(())
}
