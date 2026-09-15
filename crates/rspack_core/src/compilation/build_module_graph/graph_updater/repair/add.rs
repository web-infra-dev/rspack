use std::sync::Arc;

use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_error::Result;
use rspack_util::fx_hash::FxHashSet;

use super::{
  TaskContext,
  build::{BuildResultTask, BuildTask, ModuleBuildResult},
  lazy::process_unlazy_dependencies,
  process_dependencies::ProcessDependenciesTask,
};
use crate::{
  BoxModule, BuildContext, DependencyRef, ModuleIdentifier, ParserCreatedModuleConnection,
  compilation::build_module_graph::ForwardedIdSet,
  module_graph::{ModuleGraph, ModuleGraphModule},
  utils::task_loop::{Task, TaskResult, TaskType},
};

#[derive(Debug)]
pub struct AddTask {
  pub build_context: Arc<BuildContext>,
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub module: BoxModule,
  pub module_graph_module: Box<ModuleGraphModule>,
  pub dependencies: Vec<DependencyRef>,
  pub from_unlazy: bool,
}

#[async_trait::async_trait]
impl Task<TaskContext> for AddTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }
  async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let Self {
      build_context,
      original_module_identifier,
      module,
      module_graph_module,
      dependencies,
      from_unlazy,
    } = *self;
    let module_identifier = module.identifier();

    // reuse module for self referenced module
    if module.as_self_module().is_some() {
      let issuer = module_graph_module
        .issuer()
        .identifier()
        .expect("self module should have issuer");

      set_resolved_module(
        &mut context.artifact.module_graph,
        original_module_identifier,
        dependencies,
        *issuer,
      )?;

      return Ok(vec![]);
    }

    let forwarded_ids = ForwardedIdSet::from_dependencies(&dependencies);

    // reuse module if module is already added by other dependency
    if context
      .artifact
      .module_graph
      .module_graph_module_by_identifier(&module_identifier)
      .is_some()
    {
      set_resolved_module(
        &mut context.artifact.module_graph,
        original_module_identifier,
        dependencies,
        module_identifier,
      )?;

      if from_unlazy {
        context
          .artifact
          .affected_modules
          .mark_as_add(&module_identifier);
      }

      if context
        .artifact
        .module_graph
        .module_by_identifier(&module_identifier)
        .is_some()
      {
        if context
          .artifact
          .module_to_lazy_make
          .has_lazy_dependencies(&module_identifier)
          && !forwarded_ids.is_empty()
        {
          if let Some(task) = process_unlazy_dependencies(
            &context.artifact.module_to_lazy_make,
            &mut context.artifact.module_graph,
            forwarded_ids,
            module_identifier,
          ) {
            return Ok(vec![Box::new(task)]);
          }
          return Ok(vec![]);
        }
      } else {
        let pending_forwarded_ids = context
          .artifact
          .module_to_lazy_make
          .pending_forwarded_ids(module_identifier);
        pending_forwarded_ids.append(forwarded_ids);
      }

      return Ok(vec![]);
    }

    let cached_module = if let Some(module_build_cache) = &context.module_build_cache {
      module_build_cache
        .restore(
          &module,
          &context.build_context.file_system_info,
          &context.value_cache_versions,
        )
        .await?
    } else {
      None
    };
    context
      .artifact
      .module_graph
      .add_module_graph_module(*module_graph_module);

    context
      .exports_info_artifact
      .new_exports_info(module_identifier);

    set_resolved_module(
      &mut context.artifact.module_graph,
      original_module_identifier,
      dependencies,
      module_identifier,
    )?;

    tracing::trace!("Module added: {module_identifier}");
    context
      .artifact
      .affected_modules
      .mark_as_add(&module_identifier);

    if let Some(module) = cached_module {
      return Ok(vec![Box::new(BuildResultTask {
        build_result: ModuleBuildResult::Cached(module),
        plugin_driver: context.build_context.plugin_driver.clone(),
        forwarded_ids,
      })]);
    }

    Ok(vec![Box::new(BuildTask {
      build_context,
      module,
      forwarded_ids,
      module_build_cache: context.module_build_cache.clone(),
    })])
  }
}

fn set_resolved_module(
  module_graph: &mut ModuleGraph,
  original_module_identifier: Option<ModuleIdentifier>,
  dependencies: Vec<DependencyRef>,
  module_identifier: ModuleIdentifier,
) -> Result<()> {
  for dependency in dependencies {
    module_graph.set_resolved_module(
      original_module_identifier,
      *dependency.id(),
      module_identifier,
    )?;
    module_graph.add_dependency_ref(dependency);
  }
  Ok(())
}

/// Prepares AddTasks for parser-created modules and removes their connections
/// from the dependencies that still need factorization.
pub(super) fn prepare_add_tasks_for_parser_created_modules(
  context: &mut TaskContext,
  modules: Vec<BoxModule>,
  module_connections: Vec<ParserCreatedModuleConnection>,
  process_dependencies: &mut ProcessDependenciesTask,
) -> Vec<Box<dyn Task<TaskContext>>> {
  let issuer = process_dependencies.original_module_identifier;
  let dependencies_to_process = process_dependencies
    .dependencies
    .iter()
    .copied()
    .collect::<FxHashSet<_>>();
  let mut resolved_dependencies = FxHashSet::default();
  let mut dependencies_by_module: IdentifierMap<Vec<DependencyRef>> = IdentifierMap::default();
  let mut connected_modules = IdentifierSet::default();
  let mut modules_by_identifier = IdentifierMap::default();
  for module in modules {
    modules_by_identifier
      .entry(module.identifier())
      .or_insert(module);
  }

  for ParserCreatedModuleConnection {
    module_identifier,
    factorize_info,
  } in module_connections
  {
    connected_modules.insert(module_identifier);
    // Lazy dependencies are processed later, without retaining parser-created modules.
    if !modules_by_identifier.contains_key(&module_identifier)
      || !factorize_info
        .related_dep_ids()
        .iter()
        .all(|id| dependencies_to_process.contains(id))
    {
      continue;
    }
    let dependencies = dependencies_by_module.entry(module_identifier).or_default();
    for id in factorize_info.related_dep_ids() {
      resolved_dependencies.insert(*id);
      context.artifact.affected_dependencies.mark_as_add(id);
      dependencies.push(
        context
          .artifact
          .module_graph
          .dependency_ref_by_id(id)
          .clone(),
      );
    }
    context.artifact.record_factorization(factorize_info);
  }

  let mut tasks: Vec<Box<dyn Task<TaskContext>>> = vec![];
  for (module_identifier, module) in modules_by_identifier {
    let dependencies = dependencies_by_module.remove(&module_identifier);
    if dependencies.is_none() && connected_modules.contains(&module_identifier) {
      continue;
    }
    let mut module_graph_module = ModuleGraphModule::new(module_identifier);
    module_graph_module.set_issuer_if_unset(Some(issuer));
    tasks.push(Box::new(AddTask {
      build_context: context.build_context.clone(),
      original_module_identifier: Some(issuer),
      module,
      module_graph_module: Box::new(module_graph_module),
      dependencies: dependencies.unwrap_or_default(),
      from_unlazy: false,
    }));
  }
  process_dependencies
    .dependencies
    .retain(|id| !resolved_dependencies.contains(id));
  tasks
}
