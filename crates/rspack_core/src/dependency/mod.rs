mod entry;
pub use entry::*;
mod code_generatable;
pub use code_generatable::*;

mod commonjs;
pub use commonjs::*;
mod esm;
pub use esm::*;
mod css;
pub use css::*;
mod hmr;
use std::{any::Any, fmt::Debug, hash::Hash};

use dyn_clone::{clone_trait_object, DynClone};
pub use hmr::*;

use crate::{AsAny, DynEq, DynHash, ErrorSpan, ModuleIdentifier};

// Used to describe dependencies' types, see webpack's `type` getter in `Dependency`
// Note: This is almost the same with the old `ResolveKind`
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DependencyType {
  #[default]
  Unknown,
  Entry,
  // Harmony import
  EsmImport,
  // import()
  DynamicImport,
  // cjs require
  CjsRequire,
  // module.hot.accept
  ModuleHotAccept,
  // css url()
  CssUrl,
  // css @import
  CssImport,
}

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DependencyCategory {
  #[default]
  Unknown,
  Esm,
  CommonJS,
  Url,
  CssImport,
}

pub trait Dependency:
  CodeGeneratable + AsModuleDependency + AsAny + DynHash + DynClone + DynEq + Send + Sync + Debug
{
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier>;
  fn set_parent_module_identifier(&mut self, _module_identifier: Option<ModuleIdentifier>) {
    // noop
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Unknown
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Unknown
  }
}

impl Dependency for Box<dyn Dependency> {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    (**self).parent_module_identifier()
  }

  fn set_parent_module_identifier(&mut self, module_identifier: Option<ModuleIdentifier>) {
    (**self).set_parent_module_identifier(module_identifier)
  }

  fn category(&self) -> &DependencyCategory {
    (**self).category()
  }

  fn dependency_type(&self) -> &DependencyType {
    (**self).dependency_type()
  }
}

impl CodeGeneratable for Box<dyn Dependency> {
  fn generate(&self, code_generatable_context: &CodeGeneratableContext) -> CodeGeneratableResult {
    (**self).generate(code_generatable_context)
  }
}

impl PartialEq for dyn Dependency + '_ {
  fn eq(&self, other: &Self) -> bool {
    self.dyn_eq(other.as_any())
  }
}

impl Eq for dyn Dependency + '_ {}

impl Hash for dyn Dependency + '_ {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.dyn_hash(state)
  }
}

pub trait AsModuleDependency {
  fn as_module_dependency(&self) -> Option<&dyn ModuleDependency> {
    None
  }
}

impl AsModuleDependency for Box<dyn Dependency> {
  fn as_module_dependency(&self) -> Option<&dyn ModuleDependency> {
    None
  }
}

impl<T: ModuleDependency> AsModuleDependency for T {
  fn as_module_dependency(&self) -> Option<&dyn ModuleDependency> {
    Some(self)
  }
}

pub trait ModuleDependency: Dependency {
  fn request(&self) -> &str;
  fn user_request(&self) -> &str;
  fn span(&self) -> Option<&ErrorSpan>;
}

impl ModuleDependency for Box<dyn ModuleDependency> {
  fn request(&self) -> &str {
    (**self).request()
  }

  fn user_request(&self) -> &str {
    (**self).user_request()
  }

  fn span(&self) -> Option<&ErrorSpan> {
    (**self).span()
  }
}

impl Dependency for Box<dyn ModuleDependency> {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    (**self).parent_module_identifier()
  }

  fn set_parent_module_identifier(&mut self, module_identifier: Option<ModuleIdentifier>) {
    (**self).set_parent_module_identifier(module_identifier)
  }

  fn category(&self) -> &DependencyCategory {
    (**self).category()
  }

  fn dependency_type(&self) -> &DependencyType {
    (**self).dependency_type()
  }
}

impl CodeGeneratable for Box<dyn ModuleDependency> {
  fn generate(&self, code_generatable_context: &CodeGeneratableContext) -> CodeGeneratableResult {
    (**self).generate(code_generatable_context)
  }
}

impl PartialEq for dyn ModuleDependency + '_ {
  fn eq(&self, other: &Self) -> bool {
    self.dyn_eq(other.as_any())
  }
}

impl Eq for dyn ModuleDependency + '_ {}

impl Hash for dyn ModuleDependency + '_ {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.dyn_hash(state)
  }
}

impl dyn Dependency + '_ {
  pub fn downcast_ref<D: Any>(&self) -> Option<&D> {
    self.as_any().downcast_ref::<D>()
  }

  pub fn downcast_mut<D: Any>(&mut self) -> Option<&mut D> {
    self.as_any_mut().downcast_mut::<D>()
  }
}

clone_trait_object!(Dependency);
clone_trait_object!(ModuleDependency);

pub type BoxModuleDependency = Box<dyn ModuleDependency>;
pub type BoxDependency = Box<dyn Dependency>;
