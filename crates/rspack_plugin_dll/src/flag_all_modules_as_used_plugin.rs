use rspack_collections::IdentifierSet;
use rspack_core::{
  get_entry_runtime, merge_runtime, ApplyContext, BoxModule, Compilation,
  CompilationOptimizeDependencies, CompilerOptions, FactoryMeta, ModuleFactoryCreateData,
  NormalModuleCreateData, NormalModuleFactoryModule, Plugin, PluginContext, RuntimeSpec,
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
      .normal_module_factory_hooks
      .module
      .tap(nmf_module::new(self));

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
    if let Some(mgm) = mg.module_graph_module_by_identifier_mut(&module_id) {
      mgm.add_concatenation_bail_reason(&self.explanation);
    };
  }

  Ok(None)
}

#[plugin_hook(NormalModuleFactoryModule for FlagAllModulesAsUsedPlugin)]
async fn nmf_module(
  &self,
  _data: &mut ModuleFactoryCreateData,
  _create_date: &mut NormalModuleCreateData,
  module: &mut BoxModule,
) -> Result<()> {
  // set all modules have effects. To avoid any module remove by tree shaking.
  // see: https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/FlagAllModulesAsUsedPlugin.js#L43-L47
  module.set_factory_meta(FactoryMeta {
    side_effect_free: Some(false),
  });

  Ok(())
}
