use rspack_collections::IdentifierMap;
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  make::repair::{process_dependencies::ProcessDependenciesTask, MakeTaskContext},
  task_loop::{Task, TaskResult, TaskType},
  DependencyId, ModuleIdentifier,
};

#[derive(Debug, Default)]
pub struct ForwardedIdSet(FxHashSet<Atom>);

impl ForwardedIdSet {
  pub fn new(set: FxHashSet<Atom>) -> Self {
    Self(set)
  }

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
pub struct LazyMake {
  pub forward_id: Option<Atom>,
  pub kind: LazyMakeKind,
}

#[derive(Debug, Default)]
pub enum LazyMakeKind {
  #[default]
  Eager,
  Lazy {
    until: Option<Atom>,
  },
}

impl LazyMake {
  pub fn is_lazy(&self) -> bool {
    matches!(self.kind, LazyMakeKind::Lazy { .. })
  }
}

#[derive(Debug, Default)]
pub struct LazyDependencies {
  forward_id_to_request: FxHashMap<Atom, Atom>,
  request_to_dependencies: FxHashMap<Atom, FxHashSet<DependencyId>>,
}

impl LazyDependencies {
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
    forwarded_ids: &'a ForwardedIdSet,
  ) -> impl Iterator<Item = DependencyId> + use<'a, '_> {
    forwarded_ids
      .0
      .iter()
      .filter_map(|forward_id| {
        self
          .forward_id_to_request
          .get(forward_id)
          .and_then(|request| self.request_to_dependencies.get(request))
      })
      .flatten()
      .copied()
  }
}

#[derive(Debug)]
pub enum HasLazyDependencies {
  Maybe(ForwardedIdSet),
  Has(LazyDependencies),
}

impl HasLazyDependencies {
  pub fn expect_has(&self, msg: &str) -> &LazyDependencies {
    if let HasLazyDependencies::Has(lazy_dependencies_info) = self {
      lazy_dependencies_info
    } else {
      panic!("{}", msg);
    }
  }

  pub fn expect_maybe_mut(&mut self, msg: &str) -> &mut ForwardedIdSet {
    if let HasLazyDependencies::Maybe(pending_forward_ids) = self {
      pending_forward_ids
    } else {
      panic!("{}", msg);
    }
  }

  pub fn into_maybe(self) -> Option<ForwardedIdSet> {
    if let HasLazyDependencies::Maybe(pending_forward_ids) = self {
      Some(pending_forward_ids)
    } else {
      None
    }
  }
}

#[derive(Debug, Default)]
pub struct ModuleToLazyMake {
  module_to_lazy_dependencies: IdentifierMap<HasLazyDependencies>,
}

impl ModuleToLazyMake {
  pub fn get_lazy_dependencies(&self, module: &ModuleIdentifier) -> Option<&LazyDependencies> {
    self
      .module_to_lazy_dependencies
      .get(module)
      .and_then(|info| match info {
        HasLazyDependencies::Maybe(_) => None,
        HasLazyDependencies::Has(lazy_dependencies) => Some(lazy_dependencies),
      })
  }

  pub fn update_module_lazy_dependencies(
    &mut self,
    module: ModuleIdentifier,
    to: Option<LazyDependencies>,
  ) -> Option<HasLazyDependencies> {
    match to {
      Some(lazy_dependencies) => self
        .module_to_lazy_dependencies
        .insert(module, HasLazyDependencies::Has(lazy_dependencies)),
      None => self.module_to_lazy_dependencies.remove(&module),
    }
  }

  pub fn has_lazy_dependencies(&self, module: &ModuleIdentifier) -> bool {
    self.module_to_lazy_dependencies.contains_key(module)
  }

  pub fn maybe_lazy_dependencies(&mut self, module: ModuleIdentifier) -> &mut ForwardedIdSet {
    self
      .module_to_lazy_dependencies
      .entry(module)
      .or_insert_with(|| HasLazyDependencies::Maybe(ForwardedIdSet::default()))
      .expect_maybe_mut("should be maybe lazy dependencies")
  }
}

#[derive(Debug)]
pub struct ProcessLazyDependenciesTask {
  pub forwarded_ids: ForwardedIdSet,
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
      forwarded_ids,
      original_module_identifier,
    } = *self;

    let lazy_dependencies = context
      .artifact
      .module_to_lazy_make
      .get_lazy_dependencies(&original_module_identifier)
      .expect("only module that has lazy dependencies should run into ProcessLazyDependenciesTask");
    let dependencies_to_process = lazy_dependencies
      .get_requested_lazy_dependencies(&forwarded_ids)
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
