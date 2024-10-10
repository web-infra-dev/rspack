use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{AsContextDependency, Dependency};
use rspack_core::{
  Compilation, DependencyType, ExternalRequest, ExternalType, ImportAttributes,
  RealDependencyLocation, RuntimeSpec,
};
use rspack_core::{DependencyCategory, DependencyId, DependencyTemplate};
use rspack_core::{ModuleDependency, TemplateContext, TemplateReplaceSource};
use rspack_plugin_javascript::dependency::create_resource_identifier_for_esm_dependency;
use swc_core::ecma::atoms::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ModernModuleImportDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  target_request: ExternalRequest,
  external_type: ExternalType,
  range: RealDependencyLocation,
  attributes: Option<ImportAttributes>,
  resource_identifier: String,
}

impl ModernModuleImportDependency {
  pub fn new(
    request: Atom,
    target_request: ExternalRequest,
    external_type: ExternalType,
    range: RealDependencyLocation,
    attributes: Option<ImportAttributes>,
  ) -> Self {
    let resource_identifier =
      create_resource_identifier_for_esm_dependency(request.as_str(), attributes.as_ref());
    Self {
      request,
      target_request,
      external_type,
      range,
      id: DependencyId::new(),
      attributes,
      resource_identifier,
    }
  }
}

#[cacheable_dyn]
impl Dependency for ModernModuleImportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::DynamicImport
  }

  fn get_attributes(&self) -> Option<&ImportAttributes> {
    self.attributes.as_ref()
  }

  fn range(&self) -> Option<&RealDependencyLocation> {
    Some(&self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ModernModuleImportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }
}

#[cacheable_dyn]
impl DependencyTemplate for ModernModuleImportDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    let request_and_external_type = match &self.target_request {
      ExternalRequest::Single(request) => (Some(request), &self.external_type),
      ExternalRequest::Map(map) => (map.get(&self.external_type), &self.external_type),
    };

    if let Some(request_and_external_type) = request_and_external_type.0 {
      source.replace(
        self.range.start,
        self.range.end,
        format!(
          "import({})",
          serde_json::to_string(request_and_external_type.primary())
            .expect("invalid json to_string")
        )
        .as_str(),
        None,
      );
    }
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }
}

impl AsContextDependency for ModernModuleImportDependency {}
