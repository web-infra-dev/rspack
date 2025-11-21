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
  BuildDependency, Compilation, ModuleProfile,
  utils::task_loop::{Task, run_task_loop},
};

pub async fn repair(
  compilation: &Compilation,
  mut artifact: BuildModuleGraphArtifact,
  build_dependencies: HashSet<BuildDependency>,
) -> Result<BuildModuleGraphArtifact> {
  let module_graph = artifact.get_module_graph_mut();
  let mut grouped_deps = HashMap::default();
  for (dep_id, parent_module_identifier) in build_dependencies {
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
          let dependency = module_graph
            .dependency_by_id(&dep_id)
            .expect("dependency not found");
          let current_profile = compilation.options.profile.then(ModuleProfile::default);
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
            current_profile,
            resolver_factory: compilation.resolver_factory.clone(),
            from_unlazy: false,
          }) as Box<dyn Task<TaskContext>>
        })
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();

  let mut ctx = TaskContext::new(compilation, artifact);
  run_task_loop(&mut ctx, init_tasks).await?;
  Ok(ctx.artifact)
}
