use rspack_collections::IdentifierSet;
use rspack_core::{
  get_entry_runtime, merge_runtime, ApplyContext, BoxModule, Compilation, CompilationId,
  CompilationOptimizeDependencies, CompilationSucceedModule, CompilerId, CompilerOptions,
  FactoryMeta, Plugin, PluginContext, RuntimeSpec,
};
use rspack_error::Result;
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

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));

    ctx
      .context
      .compilation_hooks
      .succeed_module
      .tap(succeed_module::new(self));

    Ok(())
  }
}

#[plugin_hook(CompilationOptimizeDependencies for FlagAllModulesAsUsedPlugin)]
fn optimize_dependencies(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let entries = &compilation.entries;

  let runtime = compilation
    .entries
    .iter()
    .map(|(name, entry_data)| get_entry_runtime(name, &entry_data.options, entries))
    .fold(RuntimeSpec::default(), |a, b| merge_runtime(&a, &b));

  let mut mg = compilation.get_module_graph_mut();

  let module_id_list: IdentifierSet = mg.modules().keys().copied().collect();

  for module_id in module_id_list {
    let exports_info = mg.get_exports_info(&module_id);
    exports_info.set_used_in_unknown_way(&mut mg, Some(&runtime));
  }

  Ok(None)
}

#[plugin_hook(CompilationSucceedModule for FlagAllModulesAsUsedPlugin)]
async fn succeed_module(
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
