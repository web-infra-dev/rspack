use dyn_clone::clone_trait_object;
use rspack_cacheable::cacheable_dyn;
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap, FxHashSet};

use super::{Dependency, FactorizeInfo};
use crate::{DependencyCondition, DependencyId, ErrorSpan};

#[derive(Debug, Default)]
pub enum LazyMake {
  #[default]
  Eager,
  LazyUntil {
    forward_name: Option<Atom>,
  },
}

#[derive(Debug, Default)]
pub struct LazyDependenciesInfo {
  forward_name_to_request: FxHashMap<Atom, Atom>,
  request_to_dependencies: FxHashMap<Atom, FxHashSet<DependencyId>>,
}

impl LazyDependenciesInfo {
  pub fn is_empty(&self) -> bool {
    self.request_to_dependencies.is_empty()
  }

  pub fn insert(&mut self, request: Atom, forward_name: Option<Atom>, dependency_id: DependencyId) {
    if let Some(forward_name) = forward_name {
      self
        .forward_name_to_request
        .insert(forward_name, request.clone());
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

  pub fn get_requested_lazy_dependencies(
    &self,
    forward_name: &Atom,
  ) -> Option<&FxHashSet<DependencyId>> {
    self
      .forward_name_to_request
      .get(forward_name)
      .and_then(|request| self.request_to_dependencies.get(request))
  }
}

#[cacheable_dyn]
pub trait ModuleDependency: Dependency {
  fn request(&self) -> &str;

  fn user_request(&self) -> &str {
    self.request()
  }

  /// Span for precise source location.
  /// For example: the source node in an `ImportDeclaration`.
  /// This is only intended used to display better diagnostics.
  /// So it might not be precise as it is using [crate::Dependency::span] as the default value.
  fn source_span(&self) -> Option<ErrorSpan> {
    self
      .range()
      .map(|range| ErrorSpan::new(range.start, range.end))
  }

  fn weak(&self) -> bool {
    matches!(self.lazy(), LazyMake::LazyUntil { .. })
  }

  fn lazy(&self) -> LazyMake {
    LazyMake::Eager
  }

  fn unset_lazy(&mut self) {}

  fn forward_name(&self) -> Option<Atom> {
    None
  }

  fn get_optional(&self) -> bool {
    false
  }

  fn get_condition(&self) -> Option<DependencyCondition> {
    None
  }

  fn factorize_info(&self) -> &FactorizeInfo;
  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo;
}

clone_trait_object!(ModuleDependency);

pub trait AsModuleDependency {
  fn as_module_dependency(&self) -> Option<&dyn ModuleDependency> {
    None
  }

  fn as_module_dependency_mut(&mut self) -> Option<&mut dyn ModuleDependency> {
    None
  }
}

impl<T: ModuleDependency> AsModuleDependency for T {
  fn as_module_dependency(&self) -> Option<&dyn ModuleDependency> {
    Some(self)
  }

  fn as_module_dependency_mut(&mut self) -> Option<&mut dyn ModuleDependency> {
    Some(self)
  }
}

pub type BoxModuleDependency = Box<dyn ModuleDependency>;
