use rspack_error::Diagnostic;

use super::FactorizeInfo;
use crate::{ContextOptions, ContextTypePrefix, Dependency};

pub trait ContextDependency: Dependency {
  fn request(&self) -> &str;
  fn options(&self) -> &ContextOptions;
  fn get_context(&self) -> Option<&str>;
  fn resource_identifier(&self) -> &str;

  fn get_optional(&self) -> bool {
    false
  }

  fn type_prefix(&self) -> ContextTypePrefix;

  fn critical(&self) -> &Option<Diagnostic>;
  fn critical_mut(&mut self) -> &mut Option<Diagnostic>;

  fn factorize_info(&self) -> &FactorizeInfo;
  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo;
}

pub trait AsContextDependency {
  fn as_context_dependency(&self) -> Option<&dyn ContextDependency> {
    None
  }

  fn as_context_dependency_mut(&mut self) -> Option<&mut dyn ContextDependency> {
    None
  }
}

impl<T: ContextDependency> AsContextDependency for T {
  fn as_context_dependency(&self) -> Option<&dyn ContextDependency> {
    Some(self)
  }

  fn as_context_dependency_mut(&mut self) -> Option<&mut dyn ContextDependency> {
    Some(self)
  }
}
