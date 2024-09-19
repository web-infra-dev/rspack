use linked_hash_set::LinkedHashSet;
use rspack_collections::IdentifierSet;
use rspack_core::{
  unaffected_cache::{Mutation, Mutations},
  ApplyContext, Compilation, CompilationFinishModules, CompilerOptions, DependencyType,
  ModuleIdentifier, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug, Default)]
pub struct InferAsyncModulesPlugin;

#[plugin_hook(CompilationFinishModules for InferAsyncModulesPlugin)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  let modules: IdentifierSet = if compilation.options.new_incremental_enabled() {
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
    .options
    .new_incremental_enabled()
    .then(|| Mutations::default());

  set_async_modules(compilation, sync_modules, false, &mut mutations);
  set_async_modules(compilation, async_modules, true, &mut mutations);

  if let Some(compilation_mutations) = &mut compilation.mutations
    && let Some(mutations) = mutations
  {
    compilation_mutations.extend(mutations);
  }

  Ok(())
}

fn set_async_modules(
  compilation: &mut Compilation,
  modules: LinkedHashSet<ModuleIdentifier>,
  is_async: bool,
  mutations: &mut Option<Mutations>,
) {
  let mut uniques = IdentifierSet::from_iter(modules.iter().copied());
  let mut queue = modules;
  let mut module_graph = compilation.get_module_graph_mut();

  while let Some(module) = queue.pop_front() {
    let changed = module_graph.set_async(&module, is_async);
    if changed && let Some(mutations) = mutations {
      mutations.add(Mutation::ModuleGraphModuleSetAsync { module });
    }
    module_graph
      .get_incoming_connections(&module)
      .iter()
      .filter(|con| {
        if let Some(dep) = module_graph.dependency_by_id(&con.dependency_id) {
          matches!(
            dep.dependency_type(),
            DependencyType::EsmImport | DependencyType::EsmExport
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
