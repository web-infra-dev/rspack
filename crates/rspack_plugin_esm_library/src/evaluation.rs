use rspack_core::{
  ContextMode, DependenciesBlock, DependencyId, DependencyType, Module, ModuleGraph,
};

/// Return direct dependencies and dependencies nested in async blocks in
/// source order. Evaluation planning must see both: dynamic imports and
/// context elements are commonly stored only on an `AsyncDependenciesBlock`.
pub(crate) fn module_dependencies(
  module: &dyn Module,
  module_graph: &ModuleGraph,
) -> Vec<DependencyId> {
  let mut dependencies = module.get_dependencies().to_vec();
  let mut blocks = module.get_blocks().to_vec();
  let mut next_block = 0;

  while let Some(block_id) = blocks.get(next_block) {
    next_block += 1;
    let Some(block) = module_graph.block_by_id(block_id) else {
      continue;
    };
    dependencies.extend_from_slice(block.get_dependencies());
    blocks.extend_from_slice(block.get_blocks());
  }

  dependencies
}

pub(crate) fn is_async_evaluation_edge(dependency_type: &DependencyType) -> bool {
  matches!(
    dependency_type,
    DependencyType::DynamicImport
      | DependencyType::DynamicImportEager
      | DependencyType::DynamicImportWeak
      | DependencyType::LazyImport
      | DependencyType::NewWorker
      | DependencyType::ContextElement(rspack_core::ContextTypePrefix::Import)
  )
}

/// Whether following this dependency loads a different chunk instead of using
/// an initializer binding that is statically visible in the source chunk.
pub(crate) fn is_chunk_loading_evaluation_edge(
  source: &dyn Module,
  dependency_type: &DependencyType,
) -> bool {
  if is_async_evaluation_edge(dependency_type) {
    return true;
  }

  matches!(dependency_type, DependencyType::ContextElement(_))
    && source.as_context_module().is_some_and(|context| {
      matches!(
        context.get_context_options().mode,
        ContextMode::Lazy | ContextMode::LazyOnce | ContextMode::AsyncWeak
      )
    })
}

pub(crate) fn starts_initializer_evaluation(dependency_type: &DependencyType) -> bool {
  is_async_evaluation_edge(dependency_type)
    || matches!(
      dependency_type,
      DependencyType::CjsRequire
        | DependencyType::CjsFullRequire
        | DependencyType::CjsExportRequire
        | DependencyType::AmdRequireItem
        | DependencyType::RequireEnsureItem
        | DependencyType::ContextElement(rspack_core::ContextTypePrefix::Normal)
    )
}
