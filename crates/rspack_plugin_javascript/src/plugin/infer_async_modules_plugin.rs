use std::collections::HashSet;

use linked_hash_set::LinkedHashSet;
use rspack_core::{Compilation, DependencyType, Plugin};
use rspack_error::Result;
use rspack_identifier::Identifier;

#[derive(Debug)]
pub struct InferAsyncModulesPlugin;

#[async_trait::async_trait]
impl Plugin for InferAsyncModulesPlugin {
  fn name(&self) -> &'static str {
    "InferAsyncModulesPlugin"
  }

  async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
    // fix: mut for-in
    let mut queue = LinkedHashSet::new();
    let mut uniques = HashSet::new();

    let mut modules: Vec<Identifier> = compilation
      .module_graph
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

    let module_graph = &mut compilation.module_graph;

    while let Some(module) = queue.pop_front() {
      module_graph.set_async(&module);
      if let Some(mgm) = module_graph.module_graph_module_by_identifier(&module) {
        mgm
          .get_incoming_connections_unordered(module_graph)?
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
    }
    Ok(())
  }
}
