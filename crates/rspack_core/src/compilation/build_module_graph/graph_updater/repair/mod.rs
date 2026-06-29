pub mod add;
pub mod build;
pub mod context;
pub mod factorize;
pub mod lazy;
pub mod process_dependencies;

use rspack_error::Result;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use self::context::TaskContext;
use super::BuildModuleGraphArtifact;
use crate::{
  BoxDependency, BuildDependency, Compilation, ContextDependency, ExportsInfoArtifact,
  utils::task_loop::{Task, run_task_loop},
};

fn cacheable_resolved_module_key(dependency: &BoxDependency) -> Option<&str> {
  if let Some(module_dependency) = dependency.as_module_dependency() {
    return module_dependency
      .request()
      .starts_with('/')
      .then(|| module_dependency.resource_identifier())
      .flatten();
  }

  dependency
    .as_context_dependency()
    .and_then(|context_dependency| {
      context_dependency
        .request()
        .starts_with('/')
        .then(|| ContextDependency::resource_identifier(context_dependency))
    })
}

pub async fn repair(
  compilation: &Compilation,
  mut artifact: BuildModuleGraphArtifact,
  exports_info_artifact: ExportsInfoArtifact,
  build_dependencies: HashSet<BuildDependency>,
) -> Result<(BuildModuleGraphArtifact, ExportsInfoArtifact)> {
  let module_graph = artifact.get_module_graph_mut();
  let mut grouped_deps = HashMap::default();
  let mut resolved_absolute_request_modules = HashMap::default();
  for (dep_id, parent_module_identifier) in build_dependencies {
    let dependency = module_graph.dependency_by_id(&dep_id);
    if let Some(key) = cacheable_resolved_module_key(dependency)
      && let Some(module_identifier) = module_graph.get_resolved_module(&dep_id).copied()
      && module_graph
        .module_by_identifier(&module_identifier)
        .is_some()
    {
      resolved_absolute_request_modules.insert(key.into(), module_identifier);
    }

    grouped_deps
      .entry(parent_module_identifier)
      .or_insert(vec![])
      .push(dep_id);
  }
  let init_tasks = grouped_deps
    .into_iter()
    .flat_map(|(parent_module_identifier, dependencies)| {
      if let Some(original_module_identifier) = parent_module_identifier {
        return vec![Box::new(process_dependencies::ProcessDependenciesTask {
          original_module_identifier,
          dependencies,
          from_unlazy: false,
        }) as Box<dyn Task<TaskContext>>];
      }
      // entry dependencies
      dependencies
        .into_iter()
        .map(|dep_id| {
          let dependency = module_graph.dependency_by_id(&dep_id);
          Box::new(factorize::FactorizeTask {
            compiler_id: compilation.compiler_id(),
            compilation_id: compilation.id(),
            module_factory: compilation.get_dependency_factory(dependency),
            original_module_identifier: None,
            original_module_source: None,
            issuer: None,
            issuer_layer: None,
            original_module_context: None,
            dependencies: vec![dependency.clone()],
            resolve_options: None,
            options: compilation.options.clone(),
            resolver_factory: compilation.resolver_factory.clone(),
            from_unlazy: false,
          }) as Box<dyn Task<TaskContext>>
        })
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();

  let mut ctx = TaskContext::new(compilation, artifact, exports_info_artifact);
  ctx.resolved_absolute_request_modules = resolved_absolute_request_modules;
  run_task_loop(&mut ctx, init_tasks).await?;
  Ok((ctx.artifact, ctx.exports_info_artifact))
}
