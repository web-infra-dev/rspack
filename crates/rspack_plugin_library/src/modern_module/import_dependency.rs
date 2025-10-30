use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType, ExternalRequest,
  ExternalType, FactorizeInfo, ImportAttributes, ModuleDependency, ResourceIdentifier,
  TemplateContext, TemplateReplaceSource,
};
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
  range: DependencyRange,
  attributes: Option<ImportAttributes>,
  resource_identifier: ResourceIdentifier,
  factorize_info: FactorizeInfo,
  pub comments: Vec<(bool, String)>,
}

impl ModernModuleImportDependency {
  pub fn new(
    id: DependencyId,
    request: Atom,
    target_request: ExternalRequest,
    external_type: ExternalType,
    range: DependencyRange,
    attributes: Option<ImportAttributes>,
    comments: Vec<(bool, String)>,
  ) -> Self {
    let resource_identifier =
      create_resource_identifier_for_esm_dependency(request.as_str(), attributes.as_ref());
    Self {
      id,
      request,
      target_request,
      external_type,
      range,
      attributes,
      resource_identifier,
      factorize_info: Default::default(),
      comments,
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

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
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

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ModernModuleImportDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ModernModuleImportDependencyTemplate::template_type())
  }
}

impl AsContextDependency for ModernModuleImportDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ModernModuleImportDependencyTemplate;
impl ModernModuleImportDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ModernModuleImportDependency")
  }
}

impl DependencyTemplate for ModernModuleImportDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ModernModuleImportDependency>()
      .expect(
        "ModernModuleImportDependencyTemplate should be used for ModernModuleImportDependency",
      );

    let request_and_external_type = match &dep.target_request {
      ExternalRequest::Single(request) => (Some(request), &dep.external_type),
      ExternalRequest::Map(map) => (map.get(&dep.external_type), &dep.external_type),
    };

    if let Some(request_and_external_type) = request_and_external_type.0 {
      let attributes_str = if let Some(attributes) = &dep.attributes {
        format!(
          ", {{ with: {} }}",
          serde_json::to_string(attributes).expect("invalid json to_string")
        )
      } else {
        String::new()
      };

      source.replace(
        dep.range.start,
        dep.range.end,
        format!(
          "import({}{}{})",
          {
            let mut comments_string = String::new();

            for (line_comment, comment) in dep.comments.iter() {
              if *line_comment {
                comments_string.push_str(&format!("//{comment}\n"));
              } else {
                comments_string.push_str(&format!("/*{comment}*/ "));
              }
            }

            comments_string
          },
          serde_json::to_string(request_and_external_type.primary())
            .expect("invalid json to_string"),
          attributes_str
        )
        .as_str(),
        None,
      );
    }
  }
}
