use rayon::prelude::*;
use rspack_collections::{IdentifierLinkedSet, IdentifierMap, IdentifierSet};
use rspack_core::{
  AsyncModulesArtifact, Compilation, CompilationFinishModules, DependencyType, ExportsInfoArtifact,
  Logger, ModuleGraph, Plugin,
  incremental::{IncrementalPasses, Mutation, Mutations},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug, Default)]
pub struct InferAsyncModulesPlugin;

#[plugin_hook(CompilationFinishModules for InferAsyncModulesPlugin)]
async fn finish_modules(
  &self,
  compilation: &Compilation,
  async_modules_artifact: &mut AsyncModulesArtifact,
  _exports_info_artifact: &mut ExportsInfoArtifact,
) -> Result<()> {
  if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::FINISH_MODULES)
  {
    mutations
      .iter()
      .filter_map(|mutation| {
        if let Mutation::ModuleRemove { module } = mutation {
          Some(module)
        } else {
          None
        }
      })
      .for_each(|module| {
        async_modules_artifact.remove(module);
      });
  }

  let module_graph = compilation.get_module_graph();
  let mut sync_modules = IdentifierLinkedSet::default();
  let mut async_modules = IdentifierLinkedSet::default();
  for (module_identifier, module) in module_graph.modules() {
    let build_meta = module.build_meta();
    if build_meta.has_top_level_await {
      async_modules.insert(*module_identifier);
    } else {
      sync_modules.insert(*module_identifier);
    }
  }

  let mut mutations = compilation
    .incremental
    .mutations_writeable()
    .then(Mutations::default);

  set_sync_modules(
    module_graph,
    async_modules_artifact,
    sync_modules,
    &mut mutations,
  );
  set_async_modules(
    module_graph,
    async_modules_artifact,
    async_modules,
    &mut mutations,
  );

  if compilation
    .incremental
    .mutations_readable(IncrementalPasses::FINISH_MODULES)
    && let Some(mutations) = &mutations
  {
    let logger = compilation.get_logger("rspack.incremental.finishModules");
    logger.log(format!(
      "{} modules are updated by set_async",
      mutations.len()
    ));
  }

  if let Some(mut compilation_mutations) = compilation.incremental.mutations_write()
    && let Some(mutations) = mutations
  {
    compilation_mutations.extend(mutations);
  }

  Ok(())
}

fn set_sync_modules(
  module_graph: &ModuleGraph,
  async_modules_artifact: &mut AsyncModulesArtifact,
  modules: IdentifierLinkedSet,
  mutations: &mut Option<Mutations>,
) {
  let outgoing_connections = modules
    .iter()
    .par_bridge()
    .map(|mid| {
      (
        *mid,
        module_graph
          .get_outgoing_connections(mid)
          .filter_map(|con| module_graph.module_identifier_by_dependency_id(&con.dependency_id))
          .filter(|&out| out != mid)
          .copied()
          .collect::<Vec<_>>(),
      )
    })
    .collect::<IdentifierMap<_>>();

  let mut queue = modules;
  while let Some(module) = queue.pop_front() {
    if outgoing_connections
      .get(&module)
      .cloned()
      .unwrap_or_else(|| {
        module_graph
          .get_outgoing_connections(&module)
          .filter_map(|con| module_graph.module_identifier_by_dependency_id(&con.dependency_id))
          .filter(|&out| &module != out)
          .copied()
          .collect::<Vec<_>>()
      })
      .iter()
      .any(|out| ModuleGraph::is_async(async_modules_artifact, out))
    {
      // We can't safely reset is_async to false if there are any outgoing module is async
      continue;
    }
    // The module is_async = false will also decide its parent module is_async, so if the module is_async = false
    // is not changed, this means its parent module will be not affected, so we stop the infer at here.
    if ModuleGraph::set_async(async_modules_artifact, module, false) {
      if let Some(mutations) = mutations {
        mutations.add(Mutation::ModuleSetAsync { module });
      }
      module_graph
        .get_incoming_connections(&module)
        .filter(|con| {
          let dep = module_graph.dependency_by_id(&con.dependency_id);
          matches!(
            dep.dependency_type(),
            DependencyType::EsmImport | DependencyType::EsmExportImport
          )
        })
        .for_each(|con| {
          if let Some(id) = con.original_module_identifier {
            queue.insert(id);
          }
        });
    }
  }
}

fn set_async_modules(
  module_graph: &ModuleGraph,
  async_modules_artifact: &mut AsyncModulesArtifact,
  modules: IdentifierLinkedSet,
  mutations: &mut Option<Mutations>,
) {
  let mut queue = modules;
  let mut visited: IdentifierSet = queue.iter().copied().collect();

  while let Some(module) = queue.pop_front() {
    if ModuleGraph::set_async(async_modules_artifact, module, true)
      && let Some(mutations) = mutations
    {
      mutations.add(Mutation::ModuleSetAsync { module });
    }
    module_graph
      .get_incoming_connections(&module)
      .filter(|con| {
        let dep = module_graph.dependency_by_id(&con.dependency_id);
        matches!(
          dep.dependency_type(),
          DependencyType::EsmImport | DependencyType::EsmExportImport
        )
      })
      .for_each(|con| {
        if let Some(id) = con.original_module_identifier
          && visited.insert(id)
        {
          queue.insert(id);
        }
      });
  }
}

impl Plugin for InferAsyncModulesPlugin {
  fn name(&self) -> &'static str {
    "InferAsyncModulesPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));
    Ok(())
  }
}
