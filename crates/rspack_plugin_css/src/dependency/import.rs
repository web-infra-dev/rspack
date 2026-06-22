use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, CssExportType, CssModuleRenderCondition, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, FactorizeInfo, ModuleDependency, ResourceIdentifier,
  TemplateContext, TemplateReplaceSource, css_module_render_conditions_identifier,
  iter_css_module_render_conditions, push_css_module_identifier_part,
};

use crate::utils::source_order_to_i32;

#[cacheable]
#[derive(Debug, Clone)]
pub struct CssImportDependency {
  id: DependencyId,
  request: String,
  range: DependencyRange,
  source_order: i32,
  inherited_render_conditions: Vec<CssModuleRenderCondition>,
  render_condition: CssModuleRenderCondition,
  export_type: Option<CssExportType>,
  resource_identifier: ResourceIdentifier,
  factorize_info: FactorizeInfo,
}

impl CssImportDependency {
  pub fn new(
    request: String,
    range: DependencyRange,
    inherited_render_conditions: Vec<CssModuleRenderCondition>,
    render_condition: CssModuleRenderCondition,
    export_type: Option<CssExportType>,
  ) -> Self {
    let resource_identifier = create_resource_identifier(
      &request,
      &inherited_render_conditions,
      &render_condition,
      export_type,
    );
    Self {
      id: DependencyId::new(),
      request,
      range,
      source_order: source_order_to_i32(range.start),
      inherited_render_conditions,
      render_condition,
      export_type,
      resource_identifier,
      factorize_info: Default::default(),
    }
  }

  pub fn inherited_render_conditions(&self) -> &[CssModuleRenderCondition] {
    &self.inherited_render_conditions
  }

  pub fn render_condition(&self) -> &CssModuleRenderCondition {
    &self.render_condition
  }

  pub fn render_conditions(&self) -> impl Iterator<Item = &CssModuleRenderCondition> {
    iter_css_module_render_conditions(&self.inherited_render_conditions, &self.render_condition)
  }

  pub fn export_type(&self) -> Option<CssExportType> {
    self.export_type
  }

  pub fn has_render_conditions(&self) -> bool {
    self.render_conditions().next().is_some()
  }
}

#[cacheable_dyn]
impl Dependency for CssImportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssImport
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssImport
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn source_order(&self) -> Option<i32> {
    Some(self.source_order)
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for CssImportDependency {
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
impl DependencyCodeGeneration for CssImportDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CssImportDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CssImportDependency {}

fn create_resource_identifier(
  request: &str,
  inherited_render_conditions: &[CssModuleRenderCondition],
  render_condition: &CssModuleRenderCondition,
  export_type: Option<CssExportType>,
) -> ResourceIdentifier {
  let category = DependencyCategory::CssImport.as_str();
  let mut identifier = String::with_capacity(category.len() + request.len() + 16);
  identifier.push_str(category);
  push_css_module_identifier_part(&mut identifier, request);

  if let Some(conditions_identifier) = css_module_render_conditions_identifier(
    iter_css_module_render_conditions(inherited_render_conditions, render_condition),
  ) {
    identifier.push('|');
    identifier.push_str(&conditions_identifier);
  }

  if let Some(export_type) = export_type {
    identifier.push('|');
    identifier.push_str("exportType=");
    identifier.push_str(&export_type.to_string());
  }

  identifier.into()
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CssImportDependencyTemplate;
impl CssImportDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CssImport)
  }
}

impl DependencyTemplate for CssImportDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CssImportDependency>()
      .expect("CssImportDependencyTemplate should be used for CssImportDependency");

    source.replace_static(dep.range.start, dep.range.end, "", None);
  }
}

#[cfg(test)]
mod tests {
  use rspack_core::{CssLayer, CssModuleRenderCondition};

  use super::create_resource_identifier;

  #[test]
  fn creates_resource_identifier_with_render_conditions() {
    let first = create_resource_identifier(
      "./style.css",
      &[],
      &CssModuleRenderCondition::new(
        Some("screen".into()),
        Some("display: grid".into()),
        Some(CssLayer::Named("theme".into())),
      ),
      None,
    );
    let second = create_resource_identifier(
      "./style.css",
      &[],
      &CssModuleRenderCondition::new(
        Some("print".into()),
        Some("display: grid".into()),
        Some(CssLayer::Named("theme".into())),
      ),
      None,
    );

    assert_ne!(first, second);
  }

  #[test]
  fn creates_resource_identifier_without_delimiter_collisions() {
    let first = create_resource_identifier(
      "a|1:b",
      &[],
      &CssModuleRenderCondition::new(None, Some("x|1:y".into()), None),
      None,
    );
    let second = create_resource_identifier(
      "a",
      &[],
      &CssModuleRenderCondition::new(None, Some("1:b|x|1:y".into()), None),
      None,
    );

    assert_ne!(first, second);
  }
}
