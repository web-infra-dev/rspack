use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, Dependency, DependencyCategory, DependencyId,
  DependencyType, ExportsInfoArtifact, ExtendedReferencedExport, FactorizeInfo, ModuleDependency,
  ModuleGraph, ModuleGraphCacheArtifact, ReferencedExport, ResourceIdentifier, RuntimeSpec,
};
use swc_core::atoms::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ClientReferenceDependency {
  id: DependencyId,
  request: String,
  #[cacheable(with=AsVec<AsPreset>)]
  referenced_exports: Vec<Atom>,
  resource_identifier: ResourceIdentifier,
  factorize_info: FactorizeInfo,
}

impl ClientReferenceDependency {
  pub fn new(request: String, referenced_exports: Vec<Atom>) -> Self {
    let resource_identifier = format!("rsc-client-reference={request}").into();
    Self {
      id: DependencyId::new(),
      request,
      referenced_exports,
      resource_identifier,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for ClientReferenceDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RscClientReference
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    vec![ExtendedReferencedExport::Export(ReferencedExport::new(
      self.referenced_exports.clone(),
      false,
      false,
    ))]
  }
}

#[cacheable_dyn]
impl ModuleDependency for ClientReferenceDependency {
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

impl AsContextDependency for ClientReferenceDependency {}
impl AsDependencyCodeGeneration for ClientReferenceDependency {}
