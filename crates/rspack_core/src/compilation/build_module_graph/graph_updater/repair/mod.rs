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
  BuildDependency, Compilation, ExportsInfoArtifact, FactorizeInfo, ResourceId,
  utils::task_loop::{Task, run_task_loop},
};

pub async fn repair(
  compilation: &Compilation,
  mut artifact: BuildModuleGraphArtifact,
  exports_info_artifact: ExportsInfoArtifact,
  build_dependencies: HashSet<BuildDependency>,
) -> Result<(BuildModuleGraphArtifact, ExportsInfoArtifact)> {
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
  run_task_loop(&mut ctx, init_tasks).await?;
  sync_file_counters(&mut ctx.artifact);
  Ok((ctx.artifact, ctx.exports_info_artifact))
}

fn sync_file_counters(artifact: &mut BuildModuleGraphArtifact) {
  let active_modules = artifact
    .affected_modules
    .active()
    .copied()
    .collect::<Vec<_>>();
  let active_dependencies = artifact
    .affected_dependencies
    .active()
    .copied()
    .collect::<Vec<_>>();

  let BuildModuleGraphArtifact {
    module_graph,
    file_dependencies,
    context_dependencies,
    missing_dependencies,
    build_dependencies,
    ..
  } = artifact;
  let module_graph = &*module_graph;

  for module_id in active_modules {
    let Some(module) = module_graph.module_by_identifier(&module_id) else {
      continue;
    };
    let build_info = module.build_info();
    let resource_id = ResourceId::from(module_id);
    file_dependencies.add_files(&resource_id, &build_info.file_dependencies);
    context_dependencies.add_files(&resource_id, &build_info.context_dependencies);
    missing_dependencies.add_files(&resource_id, &build_info.missing_dependencies);
    build_dependencies.add_files(&resource_id, &build_info.build_dependencies);
  }

  for dependency_id in active_dependencies {
    let dependency = module_graph.dependency_by_id(&dependency_id);
    let Some(factorize_info) = FactorizeInfo::get_from(dependency) else {
      continue;
    };
    let resource_id = ResourceId::from(dependency_id);
    file_dependencies.add_files(&resource_id, factorize_info.file_dependencies());
    context_dependencies.add_files(&resource_id, factorize_info.context_dependencies());
    missing_dependencies.add_files(&resource_id, factorize_info.missing_dependencies());
  }
}
