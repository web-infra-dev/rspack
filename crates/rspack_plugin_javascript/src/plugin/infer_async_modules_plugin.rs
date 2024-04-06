use std::collections::HashSet;

use linked_hash_set::LinkedHashSet;
use rspack_core::{
  ApplyContext, Compilation, CompilerOptions, DependencyType, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook, AsyncSeries};
use rspack_identifier::Identifier;

#[plugin]
#[derive(Debug, Default)]
pub struct InferAsyncModulesPlugin;

#[plugin_hook(AsyncSeries<Compilation> for InferAsyncModulesPlugin)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  // fix: mut for-in
  let mut queue = LinkedHashSet::new();
  let mut uniques = HashSet::new();

  let mut modules: Vec<Identifier> = compilation
    .get_module_graph()
    .modules()
    .values()
    .filter(|m| {
      if let Some(meta) = &m.build_meta() {
        meta.has_top_level_await
      } else {
        false
      }
    })
    .map(|m| m.identifier())
    .collect();

  modules.retain(|m| queue.insert(*m));

  let mut module_graph = compilation.get_module_graph_mut();

  while let Some(module) = queue.pop_front() {
    module_graph.set_async(&module);
    module_graph
      .get_incoming_connections(&module)
      .iter()
      .filter(|con| {
        if let Some(dep) = module_graph.dependency_by_id(&con.dependency_id) {
          matches!(
            dep.dependency_type(),
            DependencyType::EsmImport(_) | DependencyType::EsmExport(_)
          )
        } else {
          false
        }
      })
      .for_each(|con| {
        if let Some(id) = &con.original_module_identifier {
          if uniques.insert(*id) {
            queue.insert(*id);
          }
        }
      });
  }
  Ok(())
}

#[async_trait::async_trait]
impl Plugin for InferAsyncModulesPlugin {
  fn name(&self) -> &'static str {
    "InferAsyncModulesPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));
    Ok(())
  }
}
