use std::sync::Arc;

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, ImportAttributes, ImportPhase, ModuleDependency, ModuleLayer, Resolve,
};
use rspack_error::Diagnostic;
use rspack_paths::InternedPathSet;

#[cacheable]
#[derive(Debug, Clone)]
pub struct DependencyOptions {
  pub request: String,
  pub phase: ImportPhase,
  pub attributes: Option<ImportAttributes>,
  pub issuer: Option<Box<str>>,
  pub issuer_layer: Option<ModuleLayer>,
  pub resolve_options: Option<Arc<Resolve>>,

  pub file_dependencies: InternedPathSet,
  pub context_dependencies: InternedPathSet,
  pub missing_dependencies: InternedPathSet,
  pub diagnostics: Vec<Diagnostic>,
}

#[cacheable]
#[derive(Debug)]
pub struct LazyCompilationDependency {
  id: DependencyId,
  options: DependencyOptions,
}

impl LazyCompilationDependency {
  pub fn new(options: DependencyOptions) -> Self {
    Self {
      id: DependencyId::new(),
      options,
    }
  }

  pub fn options(&self) -> &DependencyOptions {
    &self.options
  }
}

#[cacheable_dyn]
impl ModuleDependency for LazyCompilationDependency {
  fn request(&self) -> &str {
    &self.options.request
  }
}

impl AsDependencyCodeGeneration for LazyCompilationDependency {}
impl AsContextDependency for LazyCompilationDependency {}

#[cacheable]
#[derive(Debug)]
pub(crate) struct ReservedExternalDependency {
  id: DependencyId,
  request: String,
  layer: Option<ModuleLayer>,
  resource_identifier: String,
}

impl ReservedExternalDependency {
  pub fn new(request: String, layer: Option<ModuleLayer>) -> Self {
    let resource_identifier = match &layer {
      Some(layer) => format!("lazy external|{}:{layer}{request}", layer.len()),
      None => format!("lazy external|null|{request}"),
    };
    Self {
      id: DependencyId::new(),
      resource_identifier,
      request,
      layer,
    }
  }
}

#[cacheable_dyn]
impl Dependency for ReservedExternalDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::LazyImport
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.layer.as_ref()
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ReservedExternalDependency {
  fn request(&self) -> &str {
    &self.request
  }
}

impl AsDependencyCodeGeneration for ReservedExternalDependency {}
impl AsContextDependency for ReservedExternalDependency {}

#[cacheable_dyn]
impl Dependency for LazyCompilationDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::LazyImport
  }

  fn get_phase(&self) -> ImportPhase {
    self.options.phase
  }

  fn get_attributes(&self) -> Option<&ImportAttributes> {
    self.options.attributes.as_ref()
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::Transitive
  }
}
