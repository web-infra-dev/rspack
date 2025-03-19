use dyn_clone::clone_trait_object;
use rspack_cacheable::cacheable_dyn;

use super::{Dependency, FactorizeInfo};
use crate::{DependencyCondition, ErrorSpan};

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

  // TODO: move to ModuleGraphConnection
  fn weak(&self) -> bool {
    false
  }

  fn set_request(&mut self, _request: String) {}

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
