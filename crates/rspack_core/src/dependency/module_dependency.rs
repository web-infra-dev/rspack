use dyn_clone::clone_trait_object;
use rspack_cacheable::cacheable_dyn;
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap, FxHashSet};

use super::{Dependency, FactorizeInfo};
use crate::{DependencyCondition, DependencyId, ErrorSpan};

#[derive(Debug, Default)]
pub enum DeferedName {
  #[default]
  NotDefered,
  Defered {
    forward_name: Option<Atom>,
  },
}

#[derive(Debug, Default)]
pub struct DeferedDependenciesInfo {
  forward_name_to_defered_request: FxHashMap<Atom, Atom>,
  defered_request_to_dependencies: FxHashMap<Atom, FxHashSet<DependencyId>>,
}

impl DeferedDependenciesInfo {
  pub fn insert(&mut self, request: Atom, forward_name: Option<Atom>, dependency_id: DependencyId) {
    if let Some(forward_name) = forward_name {
      self
        .forward_name_to_defered_request
        .insert(forward_name, request.clone());
    }
    self
      .defered_request_to_dependencies
      .entry(request)
      .or_default()
      .insert(dependency_id);
  }

  pub fn defered_dependencies(&self) -> impl Iterator<Item = DependencyId> + use<'_> {
    self
      .defered_request_to_dependencies
      .values()
      .flatten()
      .copied()
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
    false
  }

  fn get_optional(&self) -> bool {
    false
  }

  fn get_condition(&self) -> Option<DependencyCondition> {
    None
  }

  fn factorize_info(&self) -> &FactorizeInfo;
  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo;

  fn forward_name(&self) -> Option<Atom> {
    None
  }

  fn defered_name(&self) -> DeferedName {
    DeferedName::NotDefered
  }
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
