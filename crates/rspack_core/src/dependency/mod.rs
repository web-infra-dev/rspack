mod entry;
pub use entry::*;
mod code_generatable;
pub use code_generatable::*;
mod context_element_dependency;
pub use context_element_dependency::*;
mod import_context_dependency;
pub use import_context_dependency::*;
mod css;
use std::{any::Any, fmt::Debug, hash::Hash};

pub use css::*;
use dyn_clone::{clone_trait_object, DynClone};

use crate::{
  AsAny, ContextMode, ContextOptions, DynEq, DynHash, ErrorSpan, ModuleGraph, ModuleGraphModule,
  ModuleIdentifier,
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
  fn referencing_module_graph_module<'m>(
    &self,
    module_graph: &'m ModuleGraph,
  ) -> Option<&'m ModuleGraphModule>;

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
  fn referencing_module_graph_module<'m>(
    &self,
    module_graph: &'m ModuleGraph,
  ) -> Option<&'m ModuleGraphModule> {
    module_graph
      .dependencies_by_module_identifier(self.parent_module_identifier()?)
      .and_then(|deps| {
        deps.iter().find_map(|dep| {
          if dep.request() == self.request() && dep.dependency_type() == self.dependency_type() {
            module_graph.module_by_dependency(dep)
          } else {
            None
          }
        })
      })
  }

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
      self
        .referencing_module_graph_module(module_graph)
        .expect("Failed to resolve module graph module")
        .module_identifier,
    )
  }
}

impl<T: ModuleDependency> ModuleDependencyExt for T {
  fn referencing_module_graph_module<'m>(
    &self,
    module_graph: &'m ModuleGraph,
  ) -> Option<&'m ModuleGraphModule> {
    let this = self as &dyn ModuleDependency;
    this.referencing_module_graph_module(module_graph)
  }

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

impl AsModuleDependency for Box<dyn Dependency> {
  fn as_module_dependency(&self) -> Option<&dyn ModuleDependency> {
    (**self).as_module_dependency()
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
