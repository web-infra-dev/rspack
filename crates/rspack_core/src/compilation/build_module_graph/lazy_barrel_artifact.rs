use rspack_cacheable::{
  cacheable,
  with::{AsMap, AsPreset, AsVec},
};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_intern::Atom;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{DependencyId, DependencyRef, ModuleIdentifier};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ForwardId {
  All,
  Id(Atom),
  Empty,
  StarReexport,
}

#[derive(Debug, Clone)]
pub enum ForwardedIdSet {
  All,
  IdSet(FxHashSet<Atom>),
}

impl ForwardedIdSet {
  pub fn empty() -> Self {
    Self::IdSet(FxHashSet::default())
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
}

pub enum LazyUntil {
  Fallback,
  NoUntil,
  Id(Atom),
  Local(Atom),
  Import,
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
  #[cacheable(with=AsVec<AsPreset>)]
  import_requests: FxHashSet<Atom>,
  #[cacheable(with=AsVec<AsPreset>)]
  deferred_reexport_ids: FxHashSet<Atom>,
  requested_all: bool,
  #[cacheable(with=AsVec<AsPreset>)]
  requested_ids: FxHashSet<Atom>,
  fallback_targets: IdentifierSet,
}

impl LazyDependencies {
  pub fn is_empty(&self) -> bool {
    self.request_to_dependencies.is_empty() && self.fallback_dependencies.is_empty()
  }

  pub fn insert(&mut self, dependency: &DependencyRef, until: LazyUntil) {
    match until {
      LazyUntil::Fallback => {
        self.fallback_dependencies.insert(*dependency.id());
      }
      LazyUntil::Local(forward_id) => {
        self.terminal_forward_ids.insert(forward_id);
      }
      until => {
        let Some(dep) = dependency.as_module_dependency() else {
          return;
        };
        let request = Atom::from(dep.request());
        match until {
          LazyUntil::Id(forward_id) => {
            // SideEffectsFlagPlugin keeps a re-export of an `import defer` namespace on the barrel.
            if dependency.get_phase().is_defer() {
              self.deferred_reexport_ids.insert(forward_id.clone());
            }
            self
              .forward_id_to_request
              .insert(forward_id, request.clone());
          }
          LazyUntil::Import => {
            self.import_requests.insert(request.clone());
          }
          _ => {}
        }
        self
          .request_to_dependencies
          .entry(request)
          .or_default()
          .insert(*dependency.id());
      }
    }
  }

  /// Only a running barrel uses its import declarations. Names it merely re-exports are redirected
  /// past it, so the declarations can wait until a local export or a deferred re-export is requested.
  pub fn can_lazy_imports(&self, forwarded_ids: &ForwardedIdSet) -> bool {
    match forwarded_ids {
      ForwardedIdSet::All => false,
      ForwardedIdSet::IdSet(ids) => {
        !ids.is_empty()
          && ids.iter().all(|id| {
            !self.terminal_forward_ids.contains(id) && !self.deferred_reexport_ids.contains(id)
          })
      }
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

  fn is_known_id(&self, id: &Atom) -> bool {
    self.terminal_forward_ids.contains(id) || self.forward_id_to_request.contains_key(id)
  }

  fn references_local(&self, ids: &FxHashSet<Atom>) -> bool {
    ids
      .iter()
      .any(|forward_id| self.terminal_forward_ids.contains(forward_id))
  }

  pub fn merge_requested_ids(&mut self, ids: ForwardedIdSet) -> Option<ForwardedIdSet> {
    if self.requested_all {
      return None;
    }
    match ids {
      ForwardedIdSet::All => {
        self.requested_all = true;
        self.requested_ids.clear();
        Some(ForwardedIdSet::All)
      }
      ForwardedIdSet::IdSet(ids) => {
        let new_ids = ids
          .into_iter()
          .filter(|id| self.requested_ids.insert(id.clone()))
          .collect::<FxHashSet<_>>();
        (!new_ids.is_empty()).then_some(ForwardedIdSet::IdSet(new_ids))
      }
    }
  }

  pub fn star_reexport_ids(
    &mut self,
    target: ModuleIdentifier,
    eager_on_local_reference: bool,
  ) -> ForwardedIdSet {
    self.fallback_targets.insert(target);
    if self.requested_all
      || (eager_on_local_reference && self.references_local(&self.requested_ids))
    {
      return ForwardedIdSet::All;
    }
    ForwardedIdSet::IdSet(
      self
        .requested_ids
        .iter()
        .filter(|id| !self.is_known_id(id))
        .cloned()
        .collect(),
    )
  }

  pub fn fallback_targets(&self) -> &IdentifierSet {
    &self.fallback_targets
  }

  pub fn requested_lazy_dependencies(
    &self,
    forwarded_ids: &ForwardedIdSet,
    eager_on_local_reference: bool,
  ) -> (FxHashSet<DependencyId>, Option<ForwardedIdSet>) {
    let ids = match forwarded_ids {
      ForwardedIdSet::All => {
        return (
          self.all_lazy_dependencies().collect(),
          Some(ForwardedIdSet::All),
        );
      }
      ForwardedIdSet::IdSet(ids) => ids,
    };
    if eager_on_local_reference && self.references_local(ids) {
      return (
        self.all_lazy_dependencies().collect(),
        Some(ForwardedIdSet::All),
      );
    }

    let mut dependencies = FxHashSet::default();
    let mut unknown_ids = FxHashSet::default();
    let take = |request: &Atom, dependencies: &mut FxHashSet<DependencyId>| {
      if let Some(deps) = self.request_to_dependencies.get(request) {
        dependencies.extend(deps.iter().copied());
      }
    };
    for id in ids {
      if self.terminal_forward_ids.contains(id) {
        for request in &self.import_requests {
          take(request, &mut dependencies);
        }
      } else if let Some(request) = self.forward_id_to_request.get(id) {
        take(request, &mut dependencies);
        if self.deferred_reexport_ids.contains(id) {
          for request in &self.import_requests {
            take(request, &mut dependencies);
          }
        }
      } else if !self.fallback_dependencies.is_empty() {
        dependencies.extend(self.fallback_dependencies.iter().copied());
        unknown_ids.insert(id.clone());
      }
    }
    (
      dependencies,
      (!unknown_ids.is_empty()).then_some(ForwardedIdSet::IdSet(unknown_ids)),
    )
  }
}

#[derive(Debug)]
pub enum HasLazyDependencies {
  Pending(ForwardedIdSet),
  Has(Box<LazyDependencies>),
}

#[derive(Debug, Default)]
pub struct ModuleToLazyMake {
  /// Expand a barrel before its local exports make its factory observable.
  /// Enabled by lazy compilation to avoid changing that factory on activation.
  pub eager_on_local_reference: bool,
  module_to_lazy_dependencies: IdentifierMap<HasLazyDependencies>,
  updated_modules: IdentifierSet,
}

impl ModuleToLazyMake {
  pub fn get_lazy_dependencies(&self, module: &ModuleIdentifier) -> Option<&LazyDependencies> {
    self
      .module_to_lazy_dependencies
      .get(module)
      .and_then(|info| match info {
        HasLazyDependencies::Pending(_) => None,
        HasLazyDependencies::Has(lazy_dependencies) => Some(&**lazy_dependencies),
      })
  }

  pub fn get_lazy_dependencies_mut(
    &mut self,
    module: &ModuleIdentifier,
  ) -> Option<&mut LazyDependencies> {
    self
      .module_to_lazy_dependencies
      .get_mut(module)
      .and_then(|info| match info {
        HasLazyDependencies::Pending(_) => None,
        HasLazyDependencies::Has(lazy_dependencies) => Some(&mut **lazy_dependencies),
      })
  }

  pub fn update_module_lazy_dependencies(
    &mut self,
    module: ModuleIdentifier,
    to: Option<LazyDependencies>,
  ) -> Option<HasLazyDependencies> {
    match to {
      Some(lazy_dependencies) => self.module_to_lazy_dependencies.insert(
        module,
        HasLazyDependencies::Has(Box::new(lazy_dependencies)),
      ),
      None => self.module_to_lazy_dependencies.remove(&module),
    }
  }

  /// Modules whose lazy state changed without a rebuild, so the persistent cache must save them again.
  pub fn updated_modules(&self) -> &IdentifierSet {
    &self.updated_modules
  }

  pub fn mark_updated(&mut self, module: ModuleIdentifier) {
    self.updated_modules.insert(module);
  }

  pub fn revoke_module(&mut self, module: &ModuleIdentifier) {
    self.updated_modules.remove(module);
  }

  pub fn reset_updated_modules(&mut self) {
    self.updated_modules.clear();
  }

  pub fn take_pending_forwarded_ids(
    &mut self,
    module: &ModuleIdentifier,
  ) -> Option<ForwardedIdSet> {
    if !matches!(
      self.module_to_lazy_dependencies.get(module),
      Some(HasLazyDependencies::Pending(_))
    ) {
      return None;
    }
    match self.module_to_lazy_dependencies.remove(module) {
      Some(HasLazyDependencies::Pending(forwarded_ids)) => Some(forwarded_ids),
      _ => None,
    }
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
