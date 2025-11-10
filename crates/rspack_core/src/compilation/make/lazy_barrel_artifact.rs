use rspack_cacheable::{
  cacheable,
  with::{AsMap, AsPreset, AsVec},
};
use rspack_collections::IdentifierMap;
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{BoxDependency, DependencyId, ModuleIdentifier};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ForwardId {
  All,
  Id(Atom),
  Empty,
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
    for dep in dependencies {
      match dep.forward_id() {
        ForwardId::All => return Self::All,
        ForwardId::Id(id) => {
          set.insert(id);
        }
        ForwardId::Empty => {}
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

pub enum LazyUntil {
  Fallback,
  NoUntil,
  Id(Atom),
  Local(Atom),
}

#[cacheable]
#[derive(Debug, Default)]
pub struct LazyDependencies {
  #[cacheable(with=AsMap<AsPreset, AsPreset>)]
  forward_id_to_request: FxHashMap<Atom, Atom>,
  #[cacheable(with=AsMap<AsPreset>)]
  request_to_dependencies: FxHashMap<Atom, FxHashSet<DependencyId>>,
  #[cacheable(with=AsVec<AsPreset>)]
  terminal_forward_ids: FxHashSet<Atom>,
  fallback_dependencies: FxHashSet<DependencyId>,
}

impl LazyDependencies {
  pub fn is_empty(&self) -> bool {
    self.request_to_dependencies.is_empty() && self.fallback_dependencies.is_empty()
  }

  pub fn insert(&mut self, dependency: &BoxDependency, until: LazyUntil) {
    if matches!(&until, LazyUntil::Fallback) {
      self.fallback_dependencies.insert(*dependency.id());
    } else if let LazyUntil::Local(forward_id) = &until {
      self.terminal_forward_ids.insert(forward_id.clone());
    } else if let Some(dep) = dependency.as_module_dependency() {
      let request = Atom::from(dep.request());
      if let LazyUntil::Id(forward_id) = until {
        self
          .forward_id_to_request
          .insert(forward_id, request.clone());
      }
      self
        .request_to_dependencies
        .entry(request)
        .or_default()
        .insert(*dependency.id());
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
        .filter(|forward_id| !self.terminal_forward_ids.contains(*forward_id))
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

  pub fn pending_forwarded_ids(&mut self, module: ModuleIdentifier) -> &mut ForwardedIdSet {
    match self
      .module_to_lazy_dependencies
      .entry(module)
      .and_modify(|info| {
        if matches!(info, HasLazyDependencies::Has(_)) {
          *info = HasLazyDependencies::Pending(ForwardedIdSet::empty())
        }
      })
      .or_insert_with(|| HasLazyDependencies::Pending(ForwardedIdSet::empty()))
    {
      HasLazyDependencies::Pending(forwarded_ids) => forwarded_ids,
      HasLazyDependencies::Has(_) => unreachable!(),
    }
  }
}
