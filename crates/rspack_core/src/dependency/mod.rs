mod entry;
pub use entry::*;
mod runtime_requirements_dependency;
pub use runtime_requirements_dependency::*;
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
mod dynamic_import;
mod require_context_dependency;
mod require_resolve_dependency;
use std::{
  any::Any,
  borrow::Cow,
  fmt::{Debug, Display},
  hash::Hash,
};

use dyn_clone::{clone_trait_object, DynClone};
pub use dynamic_import::*;
pub use require_context_dependency::RequireContextDependency;
pub use require_resolve_dependency::RequireResolveDependency;
mod static_exports_dependency;
pub use static_exports_dependency::*;

use crate::{AsAny, ContextMode, ContextOptions, ErrorSpan, ModuleGraph, ModuleIdentifier};

// Used to describe dependencies' types, see webpack's `type` getter in `Dependency`
// Note: This is almost the same with the old `ResolveKind`
#[derive(Default, Clone, PartialEq, Eq, Hash, Debug)]
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
  // new URL("./foo", import.meta.url)
  NewUrl,
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
  // css modules compose
  CssCompose,
  // context element
  ContextElement,
  // import context
  ImportContext,
  // commonjs require context
  CommonJSRequireContext,
  // require.context
  RequireContext,
  // require.resolve
  RequireResolve,
  /// wasm import
  WasmImport,
  /// wasm export import
  WasmExportImported,
  /// static exports
  StaticExports,
  Custom(Cow<'static, str>),
}

impl Display for DependencyType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DependencyType::Unknown => write!(f, "unknown"),
      DependencyType::Entry => write!(f, "entry"),
      DependencyType::EsmImport => write!(f, "esm import"),
      DependencyType::EsmExport => write!(f, "esm export"),
      DependencyType::DynamicImport => write!(f, "dynamic import"),
      DependencyType::CjsRequire => write!(f, "cjs require"),
      DependencyType::NewUrl => write!(f, "new URL()"),
      DependencyType::ImportMetaHotAccept => write!(f, "import.meta.webpackHot.accept"),
      DependencyType::ImportMetaHotDecline => write!(f, "import.meta.webpackHot.decline"),
      DependencyType::ModuleHotAccept => write!(f, "module.hot.accept"),
      DependencyType::ModuleHotDecline => write!(f, "module.hot.decline"),
      DependencyType::CssUrl => write!(f, "css url"),
      DependencyType::CssImport => write!(f, "css import"),
      DependencyType::CssCompose => write!(f, "css compose"),
      DependencyType::ContextElement => write!(f, "context element"),
      DependencyType::ImportContext => write!(f, "import context"),
      DependencyType::CommonJSRequireContext => write!(f, "commonjs require context"),
      DependencyType::RequireContext => write!(f, "require.context"),
      DependencyType::RequireResolve => write!(f, "require.resolve"),
      DependencyType::WasmImport => write!(f, "wasm import"),
      DependencyType::WasmExportImported => write!(f, "wasm export imported"),
      DependencyType::StaticExports => write!(f, "static exports"),
      DependencyType::Custom(ty) => write!(f, "custom {ty}"),
    }
  }
}

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DependencyCategory {
  #[default]
  Unknown,
  Esm,
  CommonJS,
  Url,
  CssImport,
  CssCompose,
  Wasm,
}

impl From<&str> for DependencyCategory {
  fn from(value: &str) -> Self {
    match value {
      "esm" => Self::Esm,
      "commonjs" => Self::CommonJS,
      "url" => Self::Url,
      "wasm" => Self::Wasm,
      "css-import" => Self::CssImport,
      "css-compose" => Self::CssCompose,
      _ => Self::Unknown,
    }
  }
}

impl Display for DependencyCategory {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DependencyCategory::Unknown => write!(f, "unknown"),
      DependencyCategory::Esm => write!(f, "esm"),
      DependencyCategory::CommonJS => write!(f, "commonjs"),
      DependencyCategory::Url => write!(f, "url"),
      DependencyCategory::CssImport => write!(f, "css-import"),
      DependencyCategory::CssCompose => write!(f, "css-compose"),
      DependencyCategory::Wasm => write!(f, "wasm"),
    }
  }
}

pub trait Dependency: CodeGeneratable + AsAny + DynClone + Send + Sync + Debug {
  fn id(&self) -> Option<DependencyId> {
    None
  }
  fn set_id(&mut self, _id: Option<DependencyId>) {}

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
    let parent = module_graph.parent_module_by_dependency_id(&self.id().expect("should have dependency id")).expect("Dependency does not have a parent module identifier. Maybe you are calling this in an `EntryDependency`?");
    (
      (parent, module_id, *self.category()),
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
  fn weak(&self) -> bool {
    false
  }
  // TODO should split to `ModuleDependency` and `ContextDependency`
  fn options(&self) -> Option<&ContextOptions> {
    None
  }
  fn get_optional(&self) -> bool {
    false
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

  fn weak(&self) -> bool {
    (**self).weak()
  }

  fn options(&self) -> Option<&ContextOptions> {
    (**self).options()
  }

  fn get_optional(&self) -> bool {
    (**self).get_optional()
  }
}

impl Dependency for Box<dyn ModuleDependency> {
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
      return matches!(options.mode, ContextMode::Lazy | ContextMode::LazyOnce);
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
