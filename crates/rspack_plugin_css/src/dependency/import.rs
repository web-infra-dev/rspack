use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, CssModuleRenderCondition, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, FactorizeInfo, ModuleDependency, TemplateContext,
  TemplateReplaceSource, iter_css_module_render_conditions,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct CssImportDependency {
  id: DependencyId,
  request: String,
  range: DependencyRange,
  source_order: i32,
  inherited_render_conditions: Vec<CssModuleRenderCondition>,
  render_condition: CssModuleRenderCondition,
  factorize_info: FactorizeInfo,
}

impl CssImportDependency {
  pub fn new(
    request: String,
    range: DependencyRange,
    inherited_render_conditions: Vec<CssModuleRenderCondition>,
    render_condition: CssModuleRenderCondition,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
      source_order: source_order_to_i32(range.start),
      inherited_render_conditions,
      render_condition,
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

fn source_order_to_i32(source_order: u32) -> i32 {
  source_order.try_into().unwrap_or(i32::MAX)
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
