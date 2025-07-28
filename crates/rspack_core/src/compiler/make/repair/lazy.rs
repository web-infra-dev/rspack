use rspack_collections::IdentifierMap;
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  make::repair::{process_dependencies::ProcessDependenciesTask, MakeTaskContext},
  task_loop::{Task, TaskResult, TaskType},
  BoxDependency, DependencyId, ModuleIdentifier,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ForwardId {
  All,
  Id(Atom),
}

#[derive(Debug)]
pub enum ForwardedIdSet {
  All,
  IdSet(FxHashSet<Atom>),
}

impl ForwardedIdSet {
  pub fn empty() -> Self {
    Self::IdSet(FxHashSet::default())
  }

  pub fn from_dependencies(dependencies: &[BoxDependency]) -> Self {
    let mut set = FxHashSet::default();
    for forward_id in dependencies
      .iter()
      .filter_map(|dep| dep.as_module_dependency())
      .filter_map(|dep| dep.lazy().forward_id)
    {
      match forward_id {
        ForwardId::All => return Self::All,
        ForwardId::Id(id) => {
          set.insert(id);
        }
      }
    }
    Self::IdSet(set)
  }

  pub fn append(&mut self, other: Self) {
    match self {
      Self::All => {}
      Self::IdSet(set) => match other {
        Self::All => {
          *self = Self::All;
        }
        Self::IdSet(other) => {
          set.extend(other);
        }
      },
    }
  }

  pub fn is_empty(&self) -> bool {
    match self {
      Self::All => false,
      Self::IdSet(set) => set.is_empty(),
    }
  }

  pub fn contains(&self, id: &Atom) -> bool {
    match self {
      Self::All => true,
      Self::IdSet(set) => set.contains(id),
    }
  }

  pub fn remove(&mut self, id: &Atom) -> bool {
    match self {
      Self::All => true,
      Self::IdSet(set) => set.remove(id),
    }
  }
}

#[derive(Debug, Default)]
pub struct LazyMake {
  pub forward_id: Option<ForwardId>,
  pub kind: LazyMakeKind,
}

#[derive(Debug, Default)]
pub enum LazyMakeKind {
  #[default]
  Eager,
  Lazy {
    until: Option<ForwardId>,
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
  fallback_dependencies: FxHashSet<DependencyId>,
}

impl LazyDependencies {
  pub fn is_empty(&self) -> bool {
    self.request_to_dependencies.is_empty() && self.fallback_dependencies.is_empty()
  }

  pub fn insert(&mut self, request: Atom, until: Option<ForwardId>, dependency_id: DependencyId) {
    if matches!(&until, Some(ForwardId::All)) {
      self.fallback_dependencies.insert(dependency_id);
    } else {
      if let Some(ForwardId::Id(forward_id)) = until {
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
  }

  pub fn all_lazy_dependencies(&self) -> impl Iterator<Item = DependencyId> + use<'_> {
    self
      .request_to_dependencies
      .values()
      .flatten()
      .chain(self.fallback_dependencies.iter())
      .copied()
  }

  pub fn requested_lazy_dependencies(
    &self,
    forwarded_ids: &ForwardedIdSet,
  ) -> FxHashSet<DependencyId> {
    match forwarded_ids {
      ForwardedIdSet::All => self.all_lazy_dependencies().collect(),
      ForwardedIdSet::IdSet(set) => set
        .iter()
        .flat_map(|forward_id| {
          self
            .forward_id_to_request
            .get(forward_id)
            .and_then(|request| self.request_to_dependencies.get(request))
            .unwrap_or(&self.fallback_dependencies)
        })
        .copied()
        .collect(),
    }
  }
}

#[derive(Debug)]
pub enum HasLazyDependencies {
  Pending(ForwardedIdSet),
  Has(LazyDependencies),
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
        HasLazyDependencies::Pending(_) => None,
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

  pub fn as_pending_forwarded_ids(
    &mut self,
    module: ModuleIdentifier,
  ) -> Option<&mut ForwardedIdSet> {
    match self
      .module_to_lazy_dependencies
      .entry(module)
      .or_insert_with(|| HasLazyDependencies::Pending(ForwardedIdSet::empty()))
    {
      HasLazyDependencies::Pending(forwarded_ids) => Some(forwarded_ids),
      HasLazyDependencies::Has(_) => None,
    }
  }
}

#[derive(Debug)]
pub struct ProcessUnlazyDependenciesTask {
  pub forwarded_ids: ForwardedIdSet,
  pub original_module_identifier: ModuleIdentifier,
}

#[async_trait::async_trait]
impl Task<MakeTaskContext> for ProcessUnlazyDependenciesTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }

  async fn main_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
    let module_graph =
      &mut MakeTaskContext::get_module_graph_mut(&mut context.artifact.module_graph_partial);
    let ProcessUnlazyDependenciesTask {
      forwarded_ids,
      original_module_identifier,
    } = *self;

    let lazy_dependencies = context
      .artifact
      .module_to_lazy_make
      .get_lazy_dependencies(&original_module_identifier)
      .expect("only module that has lazy dependencies should run into ProcessLazyDependenciesTask");
    let dependencies_to_process = lazy_dependencies.requested_lazy_dependencies(&forwarded_ids);
    for dep in &dependencies_to_process {
      if let Some(dep) = module_graph
        .dependency_by_id_mut(dep)
        .and_then(|dep| dep.as_module_dependency_mut())
      {
        dep.unset_lazy();
      }
    }
    return Ok(vec![Box::new(ProcessDependenciesTask {
      dependencies: dependencies_to_process.into_iter().collect(),
      original_module_identifier,
    })]);
  }
}
