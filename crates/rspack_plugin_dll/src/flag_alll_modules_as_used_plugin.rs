use rspack_collections::IdentifierSet;
use rspack_core::{
  get_entry_runtime, merge_runtime, ApplyContext, Compilation, CompilationOptimizeDependencies,
  CompilerOptions, FactoryMeta, Plugin, PluginContext, RuntimeSpec,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug, Default)]
pub struct FlagAllModulesAsUsedPlugin;

impl Plugin for FlagAllModulesAsUsedPlugin {
  fn name(&self) -> &'static str {
    "rspack:FlagAllModulesAsUsedPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .optimize_dependencies
      .tap(optimze_dependencies::new(self));

    Ok(())
  }
}

#[plugin_hook(CompilationOptimizeDependencies for FlagAllModulesAsUsedPlugin)]
fn optimze_dependencies(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let entries = &compilation.entries;

  let runtime = compilation
    .entries
    .iter()
    .map(|(name, entry_data)| get_entry_runtime(name, &entry_data.options, entries))
    .fold(RuntimeSpec::default(), |a, b| merge_runtime(&a, &b));

  let mut mg = compilation.get_module_graph_mut();
  let modules_id: IdentifierSet = mg.modules().keys().cloned().collect();

  for module_id in modules_id {
    let exports_info = mg.get_exports_info(&module_id);
    exports_info.set_used_in_unknown_way(&mut mg, Some(&runtime));
    // TODO: module_graph add extra reason
    if let Some(module) = mg.module_by_identifier_mut(&module_id) {
      module.set_factory_meta(FactoryMeta {
        side_effect_free: Some(false),
      })
    };
  }

  Ok(None)
}
