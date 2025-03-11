use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, Dependency, FactorizeInfo, InitFragmentExt, InitFragmentKey,
  InitFragmentStage, NormalInitFragment,
};
use rspack_core::{Compilation, DependencyType, ExternalRequest, ExternalType, RuntimeSpec};
use rspack_core::{DependencyCategory, DependencyId, DependencyTemplate};
use rspack_core::{ModuleDependency, TemplateContext, TemplateReplaceSource};
use rspack_plugin_javascript::dependency::create_resource_identifier_for_esm_dependency;
use swc_core::ecma::atoms::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ModernModuleReexportStarExternalDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  target_request: ExternalRequest,
  external_type: ExternalType,
  resource_identifier: String,
  factorize_info: FactorizeInfo,
}

impl ModernModuleReexportStarExternalDependency {
  pub fn new(
    id: DependencyId,
    request: Atom,
    target_request: ExternalRequest,
    external_type: ExternalType,
  ) -> Self {
    let resource_identifier = create_resource_identifier_for_esm_dependency(request.as_str(), None);
    Self {
      id,
      request,
      target_request,
      external_type,
      resource_identifier,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for ModernModuleReexportStarExternalDependency {
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

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ModernModuleReexportStarExternalDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyTemplate for ModernModuleReexportStarExternalDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let request = match &self.target_request {
      ExternalRequest::Single(request) => Some(request),
      ExternalRequest::Map(map) => map.get(&self.external_type),
    };

    if let Some(request) = request {
      let chunk_init_fragments = code_generatable_context.chunk_init_fragments();
      chunk_init_fragments.push(
        NormalInitFragment::new(
          format!(
            "export * from {};\n",
            serde_json::to_string(request.primary()).expect("invalid json to_string")
          ),
          InitFragmentStage::StageESMImports,
          0,
          InitFragmentKey::Const(format!("modern_module_reexport_star_{}", self.request)),
          None,
        )
        .boxed(),
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

impl AsContextDependency for ModernModuleReexportStarExternalDependency {}
