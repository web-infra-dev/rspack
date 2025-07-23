use rspack_error::Result;
use rustc_hash::FxHashSet;

use super::{build::BuildTask, MakeTaskContext};
use crate::{
  make::repair::process_dependencies::ProcessDependenciesTask,
  module_graph::{ModuleGraph, ModuleGraphModule},
  utils::task_loop::{Task, TaskResult, TaskType},
  BoxDependency, Module, ModuleIdentifier, ModuleProfile,
};

#[derive(Debug)]
pub struct AddTask {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub module: Box<dyn Module>,
  pub module_graph_module: Box<ModuleGraphModule>,
  pub dependencies: Vec<BoxDependency>,
  pub current_profile: Option<Box<ModuleProfile>>,
}

#[async_trait::async_trait]
impl Task<MakeTaskContext> for AddTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }
  async fn main_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
    let module_identifier = self.module.identifier();
    let artifact = &mut context.artifact;
    let module_graph =
      &mut MakeTaskContext::get_module_graph_mut(&mut artifact.module_graph_partial);

    // reuse module for self referenced module
    if self.module.as_self_module().is_some() {
      let issuer = self
        .module_graph_module
        .issuer()
        .identifier()
        .expect("self module should have issuer");

      set_resolved_module(
        module_graph,
        self.original_module_identifier,
        self.dependencies,
        *issuer,
      )?;

      return Ok(vec![]);
    }

    let forward_names = self
      .dependencies
      .iter()
      .filter_map(|dep| dep.as_module_dependency())
      .filter_map(|dep| dep.forward_name())
      .collect::<FxHashSet<_>>();
    dbg!(&forward_names);
    // reuse module if module is already added by other dependency
    if module_graph
      .module_graph_module_by_identifier(&module_identifier)
      .is_some()
    {
      set_resolved_module(
        module_graph,
        self.original_module_identifier,
        self.dependencies,
        module_identifier,
      )?;
      // 2. revisit
      // get the deferred named reexport dependencies of the barrel file, and revisit
      // upgrade deferred named reexport dependencies from weak to strong
      dbg!(module_identifier, &context.module_to_lazy_dependencies);
      if module_graph
        .module_by_identifier(&module_identifier)
        .is_some()
      {
        if let Some(lazy_dependencies_info) =
          context.module_to_lazy_dependencies.get(&module_identifier)
        {
          let dependencies_to_process = forward_names
            .into_iter()
            .filter_map(|forward_name| {
              lazy_dependencies_info.get_requested_lazy_dependencies(&forward_name)
            })
            .flat_map(|deps| deps)
            .copied()
            .collect::<Vec<_>>();
          for dep in &dependencies_to_process {
            if let Some(dep) = module_graph
              .dependency_by_id_mut(dep)
              .and_then(|dep| dep.as_module_dependency_mut())
            {
              dep.unset_lazy();
            }
          }
          return Ok(vec![Box::new(ProcessDependenciesTask {
            dependencies: dependencies_to_process,
            original_module_identifier: module_identifier,
          })]);
        }
      } else {
      }

      return Ok(vec![]);
    }

    module_graph.add_module_graph_module(*self.module_graph_module);

    set_resolved_module(
      module_graph,
      self.original_module_identifier,
      self.dependencies,
      module_identifier,
    )?;

    tracing::trace!("Module added: {}", self.module.identifier());

    artifact.built_modules.insert(module_identifier);
    Ok(vec![Box::new(BuildTask {
      compiler_id: context.compiler_id,
      compilation_id: context.compilation_id,
      module: self.module,
      current_profile: self.current_profile,
      resolver_factory: context.resolver_factory.clone(),
      compiler_options: context.compiler_options.clone(),
      plugin_driver: context.plugin_driver.clone(),
      fs: context.fs.clone(),
      forward_names,
    })])
  }
}

fn set_resolved_module(
  module_graph: &mut ModuleGraph,
  original_module_identifier: Option<ModuleIdentifier>,
  dependencies: Vec<BoxDependency>,
  module_identifier: ModuleIdentifier,
) -> Result<()> {
  for dependency in dependencies {
    module_graph.set_resolved_module(
      original_module_identifier,
      *dependency.id(),
      module_identifier,
    )?;
    module_graph.add_dependency(dependency);
  }
  Ok(())
}
