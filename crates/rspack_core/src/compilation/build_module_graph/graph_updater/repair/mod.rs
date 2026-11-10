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
  BuildDependency, Compilation, ExportsInfoArtifact,
  utils::task_loop::{Task, run_task_loop},
};

pub async fn repair(
  compilation: &Compilation,
  artifact: BuildModuleGraphArtifact,
  exports_info_artifact: ExportsInfoArtifact,
  build_dependencies: HashSet<BuildDependency>,
) -> Result<(BuildModuleGraphArtifact, ExportsInfoArtifact)> {
  let mut ctx = TaskContext::new(compilation, artifact, exports_info_artifact);
  let module_graph = &ctx.artifact.module_graph;
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
          let dependency = module_graph.dependency_ref_by_id(&dep_id);
          let module_factory = compilation.get_dependency_factory(dependency.as_ref());
          let dependency = dependency.clone();
          Box::new(factorize::FactorizeTask {
            build_context: ctx.build_context.clone(),
            module_factory,
            original_module_identifier: None,
            original_module_source: None,
            issuer: None,
            issuer_layer: None,
            original_module_context: None,
            dependencies: vec![dependency],
            resolve_options: None,
            from_unlazy: false,
          }) as Box<dyn Task<TaskContext>>
        })
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>();

  run_task_loop(&mut ctx, init_tasks).await?;
  Ok((ctx.artifact, ctx.exports_info_artifact))
}
