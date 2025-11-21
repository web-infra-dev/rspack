use dyn_clone::clone_trait_object;
use rspack_cacheable::cacheable_dyn;

use super::{Dependency, FactorizeInfo};
use crate::DependencyCondition;

#[cacheable_dyn]
pub trait ModuleDependency: Dependency {
  fn request(&self) -> &str;

  fn user_request(&self) -> &str {
    self.request()
  }

  fn weak(&self) -> bool {
    self.lazy().is_some()
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
