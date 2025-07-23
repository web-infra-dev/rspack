use rspack_util::atom::Atom;
use rustc_hash::FxHashSet;

use crate::{
  make::repair::{process_dependencies::ProcessDependenciesTask, MakeTaskContext},
  task_loop::{Task, TaskResult, TaskType},
  LazyDependenciesInfo, ModuleIdentifier,
};

#[derive(Debug)]
pub enum HasLazyDependencies {
  Maybe(FxHashSet<Atom>),
  Has(LazyDependenciesInfo),
}

impl HasLazyDependencies {
  pub fn expect_has(&self, msg: &str) -> &LazyDependenciesInfo {
    if let HasLazyDependencies::Has(lazy_dependencies_info) = self {
      lazy_dependencies_info
    } else {
      panic!("{}", msg);
    }
  }

  pub fn expect_maybe_mut(&mut self, msg: &str) -> &mut FxHashSet<Atom> {
    if let HasLazyDependencies::Maybe(pending_forward_names) = self {
      pending_forward_names
    } else {
      panic!("{}", msg);
    }
  }

  pub fn into_maybe(self) -> Option<FxHashSet<Atom>> {
    if let HasLazyDependencies::Maybe(pending_forward_names) = self {
      Some(pending_forward_names)
    } else {
      None
    }
  }
}

#[derive(Debug)]
pub struct ProcessLazyDependenciesTask {
  pub forward_names: FxHashSet<Atom>,
  pub original_module_identifier: ModuleIdentifier,
}

#[async_trait::async_trait]
impl Task<MakeTaskContext> for ProcessLazyDependenciesTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }

  async fn main_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
    let module_graph =
      &mut MakeTaskContext::get_module_graph_mut(&mut context.artifact.module_graph_partial);
    let ProcessLazyDependenciesTask {
      forward_names,
      original_module_identifier,
    } = *self;

    let lazy_dependencies = context
      .artifact
      .module_to_lazy_dependencies
      .get(&original_module_identifier)
      .and_then(|info| match info {
        HasLazyDependencies::Maybe(_) => None,
        HasLazyDependencies::Has(lazy_dependencies) => Some(lazy_dependencies),
      })
      .expect("only module that has lazy dependencies should run into ProcessLazyDependenciesTask");
    let dependencies_to_process = forward_names
      .into_iter()
      .filter_map(|forward_name| lazy_dependencies.get_requested_lazy_dependencies(&forward_name))
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
      original_module_identifier,
    })]);
  }
}
