mod entry;
pub use entry::*;
mod code_generatable;
pub use code_generatable::*;
mod context_element_dependency;
pub use context_element_dependency::*;
mod common_js_require_context_dependency;
mod const_dependency;
mod import_context_dependency;
pub use common_js_require_context_dependency::*;
pub use const_dependency::ConstDependency;
pub use import_context_dependency::*;
mod require_context_dependency;
pub use require_context_dependency::RequireContextDependency;
mod css;
use std::{any::Any, fmt::Debug, hash::Hash};

pub use css::*;
use dyn_clone::{clone_trait_object, DynClone};

use crate::{
  AsAny, ContextMode, ContextOptions, DynEq, DynHash, ErrorSpan, ModuleGraph, ModuleIdentifier,
};

// Used to describe dependencies' types, see webpack's `type` getter in `Dependency`
// Note: This is almost the same with the old `ResolveKind`
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DependencyType {
  #[default]
  Unknown,
  Entry,
  // Harmony import
  EsmImport,
  // Harmony export
  EsmExport,
  // import()
  DynamicImport,
  // cjs require
  CjsRequire,
  // import.meta.webpackHot.accept
  ImportMetaHotAccept,
  // import.meta.webpackHot.decline
  ImportMetaHotDecline,
  // module.hot.accept
  ModuleHotAccept,
  // module.hot.decline
  ModuleHotDecline,
  // css url()
  CssUrl,
  // css @import
  CssImport,
  // context element
  ContextElement,
  // import context
  ImportContext,
  // commonjs require context
  CommonJSRequireContext,
  // require.context
  RequireContext,
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
  CodeGeneratable + AsAny + DynHash + DynClone + DynEq + Send + Sync + Debug
{
  fn id(&self) -> Option<DependencyId> {
    None
  }
  fn set_id(&mut self, _id: Option<DependencyId>) {}
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

  fn get_context(&self) -> Option<&str> {
    None
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

pub trait ModuleDependencyExt {
  fn decl_mapping(
    &self,
    module_graph: &ModuleGraph,
    module_id: String,
  ) -> (
    (ModuleIdentifier, String, DependencyCategory),
    ModuleIdentifier,
  );
}

impl ModuleDependencyExt for dyn ModuleDependency + '_ {
  fn decl_mapping(
    &self,
    module_graph: &ModuleGraph,
    module_id: String,
  ) -> (
    (ModuleIdentifier, String, DependencyCategory),
    ModuleIdentifier,
  ) {
    let parent = self.parent_module_identifier().expect("Dependency does not have a parent module identifier. Maybe you are calling this in an `EntryDependency`?");
    (
      (*parent, module_id, *self.category()),
      *module_graph
        .module_identifier_by_dependency_id(&self.id().expect("should have dependency"))
        .expect("Failed to resolve module graph module"),
    )
  }
}

impl<T: ModuleDependency> ModuleDependencyExt for T {
  fn decl_mapping(
    &self,
    module_graph: &ModuleGraph,
    module_id: String,
  ) -> (
    (ModuleIdentifier, String, DependencyCategory),
    ModuleIdentifier,
  ) {
    let this = self as &dyn ModuleDependency;
    this.decl_mapping(module_graph, module_id)
  }
}

impl CodeGeneratable for Box<dyn Dependency> {
  fn generate(
    &self,
    code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
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

impl<T: ModuleDependency> AsModuleDependency for T {
  fn as_module_dependency(&self) -> Option<&dyn ModuleDependency> {
    Some(self)
  }
}

pub trait ModuleDependency: Dependency {
  fn request(&self) -> &str;
  fn user_request(&self) -> &str;
  fn span(&self) -> Option<&ErrorSpan>;
  // TODO should split to `ModuleDependency` and `ContextDependency`
  fn options(&self) -> Option<&ContextOptions> {
    None
  }
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

  fn options(&self) -> Option<&ContextOptions> {
    (**self).options()
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

  fn get_context(&self) -> Option<&str> {
    (**self).get_context()
  }

  fn id(&self) -> Option<DependencyId> {
    (**self).id()
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    (**self).set_id(id)
  }
}

impl CodeGeneratable for Box<dyn ModuleDependency> {
  fn generate(
    &self,
    code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
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

pub fn is_async_dependency(dep: &BoxModuleDependency) -> bool {
  if matches!(dep.dependency_type(), DependencyType::DynamicImport) {
    return true;
  }
  if matches!(dep.dependency_type(), DependencyType::ContextElement) {
    if let Some(options) = dep.options() {
      return matches!(
        options.mode,
        ContextMode::Lazy | ContextMode::LazyOnce | ContextMode::AsyncWeak
      );
    }
  }
  false
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct DependencyId(usize);

impl std::ops::Deref for DependencyId {
  type Target = usize;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<usize> for DependencyId {
  fn from(id: usize) -> Self {
    Self(id)
  }
}
