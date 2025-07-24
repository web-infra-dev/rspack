use std::sync::Arc;

use rspack_collections::IdentifierMap;
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  make::repair::{process_dependencies::ProcessDependenciesTask, MakeTaskContext},
  task_loop::{Task, TaskResult, TaskType},
  DependencyId, ModuleIdentifier,
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ForwardIds(Arc<Vec<Atom>>);

impl ForwardIds {
  pub fn new(ids: Arc<Vec<Atom>>) -> ForwardIds {
    Self(ids)
  }
}

#[derive(Debug)]
pub struct MergedForwardIds(FxHashSet<ForwardIds>);

impl MergedForwardIds {
  pub fn new(set: FxHashSet<ForwardIds>) -> MergedForwardIds {
    MergedForwardIds(set)
  }

  pub fn get_immediate(&self) -> ImmediateForwardIdSet {
    ImmediateForwardIdSet(
      self
        .0
        .iter()
        .flat_map(|forward_ids| forward_ids.0.first())
        .cloned()
        .collect(),
    )
  }
}

#[derive(Debug, Default)]
pub struct ImmediateForwardIdSet(FxHashSet<Atom>);

impl ImmediateForwardIdSet {
  pub fn append(&mut self, other: Self) {
    self.0.extend(other.0);
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  pub fn contains(&self, id: &Atom) -> bool {
    self.0.contains(id)
  }
}

#[derive(Debug, Default)]
pub enum LazyMake {
  #[default]
  Eager,
  LazyUntil {
    forward_id: Option<Atom>,
  },
}

#[derive(Debug, Default)]
pub struct LazyDependenciesInfo {
  forward_id_to_request: FxHashMap<Atom, Atom>,
  request_to_dependencies: FxHashMap<Atom, FxHashSet<DependencyId>>,
}

impl LazyDependenciesInfo {
  pub fn is_empty(&self) -> bool {
    self.request_to_dependencies.is_empty()
  }

  pub fn insert(&mut self, request: Atom, forward_id: Option<Atom>, dependency_id: DependencyId) {
    if let Some(forward_id) = forward_id {
      self
        .forward_id_to_request
        .insert(forward_id, request.clone());
    }
    self
      .request_to_dependencies
      .entry(request)
      .or_default()
      .insert(dependency_id);
  }

  pub fn lazy_dependencies(&self) -> impl Iterator<Item = DependencyId> + use<'_> {
    self.request_to_dependencies.values().flatten().copied()
  }

  pub fn get_requested_lazy_dependencies<'a>(
    &self,
    immediate_forward_ids: &'a ImmediateForwardIdSet,
  ) -> impl Iterator<Item = DependencyId> + use<'a, '_> {
    immediate_forward_ids
      .0
      .iter()
      .filter_map(|forward_id| {
        self
          .forward_id_to_request
          .get(forward_id)
          .and_then(|request| self.request_to_dependencies.get(request))
      })
      .flat_map(|deps| deps)
      .copied()
  }
}

#[derive(Debug)]
pub enum HasLazyDependencies {
  Maybe(ImmediateForwardIdSet),
  Has(LazyDependenciesInfo),
  KeepForward(IdentifierMap<MergedForwardIds>),
}

impl HasLazyDependencies {
  pub fn expect_has(&self, msg: &str) -> &LazyDependenciesInfo {
    if let HasLazyDependencies::Has(lazy_dependencies_info) = self {
      lazy_dependencies_info
    } else {
      panic!("{}", msg);
    }
  }

  pub fn expect_maybe_mut(&mut self, msg: &str) -> &mut ImmediateForwardIdSet {
    if let HasLazyDependencies::Maybe(pending_forward_ids) = self {
      pending_forward_ids
    } else {
      panic!("{}", msg);
    }
  }

  pub fn into_maybe(self) -> Option<ImmediateForwardIdSet> {
    if let HasLazyDependencies::Maybe(pending_forward_ids) = self {
      Some(pending_forward_ids)
    } else {
      None
    }
  }
}

#[derive(Debug)]
pub struct ProcessLazyDependenciesTask {
  pub immediate_forward_ids: ImmediateForwardIdSet,
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
      immediate_forward_ids,
      original_module_identifier,
    } = *self;

    let lazy_dependencies = context
      .artifact
      .module_to_lazy_dependencies
      .get(&original_module_identifier)
      .and_then(|info| match info {
        HasLazyDependencies::Maybe(_) | HasLazyDependencies::KeepForward(_) => None,
        HasLazyDependencies::Has(lazy_dependencies) => Some(lazy_dependencies),
      })
      .expect("only module that has lazy dependencies should run into ProcessLazyDependenciesTask");
    let dependencies_to_process = lazy_dependencies
      .get_requested_lazy_dependencies(&immediate_forward_ids)
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
