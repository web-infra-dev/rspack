use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AffectType, AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory,
  DependencyId, DependencyType, FactorizeInfo, ModuleDependency,
};

#[cacheable]
#[derive(Debug, Clone, Copy)]
pub enum DtsDependencyKind {
  Import,
  Reexport,
}

impl DtsDependencyKind {
  pub fn as_dependency_type(&self) -> DependencyType {
    match self {
      Self::Import => DependencyType::EsmImport,
      Self::Reexport => DependencyType::EsmExportImport,
    }
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct DtsDependency {
  id: DependencyId,
  request: String,
  kind: DtsDependencyKind,
  factorize_info: FactorizeInfo,
}

impl DtsDependency {
  pub fn new(request: String, kind: DtsDependencyKind) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      kind,
      factorize_info: Default::default(),
    }
  }

  pub fn kind(&self) -> DtsDependencyKind {
    self.kind
  }
}

#[cacheable_dyn]
impl Dependency for DtsDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    match self.kind {
      DtsDependencyKind::Import => &DependencyType::EsmImport,
      DtsDependencyKind::Reexport => &DependencyType::EsmExportImport,
    }
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::Transitive
  }
}

#[cacheable_dyn]
impl ModuleDependency for DtsDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsContextDependency for DtsDependency {}
impl AsDependencyCodeGeneration for DtsDependency {}
