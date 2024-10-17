use linked_hash_set::LinkedHashSet;
use rspack_collections::IdentifierSet;
use rspack_core::{
  unaffected_cache::{IncrementalPasses, Mutation, Mutations},
  ApplyContext, Compilation, CompilationFinishModules, CompilerOptions, DependencyType,
  ModuleGraph, ModuleIdentifier, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug, Default)]
pub struct InferAsyncModulesPlugin;

#[plugin_hook(CompilationFinishModules for InferAsyncModulesPlugin)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  let modules: IdentifierSet = if compilation
    .incremental
    .can_read_mutations(IncrementalPasses::INFER_ASYNC_MODULES)
  {
    compilation
      .unaffected_modules_cache
      .get_affected_modules_with_module_graph()
      .lock()
      .expect("should lock")
      .clone()
  } else {
    module_graph.modules().keys().copied().collect()
  };

  let mut sync_modules = LinkedHashSet::new();
  let mut async_modules = LinkedHashSet::new();
  for module_identifier in modules {
    let module = module_graph
      .module_by_identifier(&module_identifier)
      .expect("should have module");
    let build_meta = module.build_meta().expect("should have build meta");
    if build_meta.has_top_level_await {
      async_modules.insert(module_identifier);
    } else {
      sync_modules.insert(module_identifier);
    }
  }

  let mut mutations = compilation
    .incremental
    .can_write_mutations()
    .then(Mutations::default);

  set_sync_modules(compilation, sync_modules, &mut mutations);
  set_async_modules(compilation, async_modules, &mut mutations);

  if let Some(compilation_mutations) = compilation.incremental.mutations_write()
    && let Some(mutations) = mutations
  {
    compilation_mutations.extend(mutations);
  }

  Ok(())
}

fn set_sync_modules(
  compilation: &mut Compilation,
  modules: LinkedHashSet<ModuleIdentifier>,
  mutations: &mut Option<Mutations>,
) {
  let mut queue = modules;

  while let Some(module) = queue.pop_front() {
    let module_graph = compilation.get_module_graph();
    if module_graph
      .get_outgoing_connections(&module)
      .iter()
      .filter_map(|con| module_graph.module_identifier_by_dependency_id(&con.dependency_id))
      .any(|module| ModuleGraph::is_async(compilation, module))
    {
      // We can't safely reset is_async to false if there are any outgoing module is async
      continue;
    }
    // The module is_async will also decide its parent module is_async, so if the module is_async
    // is not changed, this means its parent module will be not affected, so we stop the infer at here.
    // This also applies to set_async_modules
    if ModuleGraph::set_async(compilation, module, false) {
      if let Some(mutations) = mutations {
        mutations.add(Mutation::ModuleSetAsync { module });
      }
      let module_graph = compilation.get_module_graph();
      module_graph
        .get_incoming_connections(&module)
        .iter()
        .filter(|con| {
          module_graph
            .dependency_by_id(&con.dependency_id)
            .map(|dep| {
              matches!(
                dep.dependency_type(),
                DependencyType::EsmImport | DependencyType::EsmExport
              )
            })
            .unwrap_or_default()
        })
        .for_each(|con| {
          if let Some(id) = &con.original_module_identifier {
            queue.insert(*id);
          }
        });
    }
  }
}

fn set_async_modules(
  compilation: &mut Compilation,
  modules: LinkedHashSet<ModuleIdentifier>,
  mutations: &mut Option<Mutations>,
) {
  let mut queue = modules;

  while let Some(module) = queue.pop_front() {
    if ModuleGraph::set_async(compilation, module, true) {
      if let Some(mutations) = mutations {
        mutations.add(Mutation::ModuleSetAsync { module });
      }
      let module_graph = compilation.get_module_graph();
      module_graph
        .get_incoming_connections(&module)
        .iter()
        .filter(|con| {
          module_graph
            .dependency_by_id(&con.dependency_id)
            .map(|dep| {
              matches!(
                dep.dependency_type(),
                DependencyType::EsmImport | DependencyType::EsmExport
              )
            })
            .unwrap_or_default()
        })
        .for_each(|con| {
          if let Some(id) = &con.original_module_identifier {
            queue.insert(*id);
          }
        });
    }
  }
}

impl Plugin for InferAsyncModulesPlugin {
  fn name(&self) -> &'static str {
    "InferAsyncModulesPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));
    Ok(())
  }
}
