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
  mut artifact: BuildModuleGraphArtifact,
  exports_info_artifact: ExportsInfoArtifact,
  build_dependencies: HashSet<BuildDependency>,
) -> Result<(BuildModuleGraphArtifact, ExportsInfoArtifact)> {
  let module_graph = artifact.get_module_graph_mut();
  let build_dependencies_len = build_dependencies.len();
  let mut grouped_deps = HashMap::default();
  for (dep_id, parent_module_identifier) in build_dependencies {
    grouped_deps
      .entry(parent_module_identifier)
      .or_insert(vec![])
      .push(dep_id);
  }
  let compiler_id = compilation.compiler_id();
  let compilation_id = compilation.id();
  let options = compilation.options.clone();
  let resolver_factory = compilation.resolver_factory.clone();
  let mut init_tasks: Vec<Box<dyn Task<TaskContext>>> = Vec::with_capacity(build_dependencies_len);
  for (parent_module_identifier, dependencies) in grouped_deps {
    if let Some(original_module_identifier) = parent_module_identifier {
      init_tasks.push(Box::new(process_dependencies::ProcessDependenciesTask {
        original_module_identifier,
        dependencies,
        from_unlazy: false,
      }));
      continue;
    }

    for dep_id in dependencies {
      let dependency = module_graph.dependency_by_id(&dep_id);
      init_tasks.push(Box::new(factorize::FactorizeTask {
        compiler_id,
        compilation_id,
        module_factory: compilation.get_dependency_factory(dependency),
        dependencies_marked: false,
        original_module_identifier: None,
        original_module_source: None,
        issuer: None,
        issuer_layer: None,
        original_module_context: None,
        dependencies: vec![dependency.clone()],
        resolve_options: None,
        options: options.clone(),
        resolver_factory: resolver_factory.clone(),
        from_unlazy: false,
      }));
    }
  }

  let mut ctx = TaskContext::new(compilation, artifact, exports_info_artifact);
  run_task_loop(&mut ctx, init_tasks).await?;
  Ok((ctx.artifact, ctx.exports_info_artifact))
}
