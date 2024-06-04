use crate::{ContextOptions, ContextTypePrefix, Dependency};

pub trait ContextDependency: Dependency {
  fn request(&self) -> &str;
  fn options(&self) -> &ContextOptions;
  fn get_context(&self) -> Option<&str>;
  fn resource_identifier(&self) -> &str;
  fn set_request(&mut self, request: String);

  fn get_optional(&self) -> bool {
    false
  }

  fn type_prefix(&self) -> ContextTypePrefix;
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
