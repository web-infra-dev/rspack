use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, RuntimeGlobals, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct CreateScriptUrlDependency {
  id: DependencyId,
  range: DependencyRange,
  range_path: DependencyRange,
}

impl CreateScriptUrlDependency {
  pub fn new(range: DependencyRange, range_path: DependencyRange) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      range_path,
    }
  }
}

#[cacheable_dyn]
impl Dependency for CreateScriptUrlDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Worker
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CreateScriptUrl
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for CreateScriptUrlDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CreateScriptUrlDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for CreateScriptUrlDependency {}
impl AsContextDependency for CreateScriptUrlDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CreateScriptUrlDependencyTemplate;

impl CreateScriptUrlDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CreateScriptUrl)
  }
}

impl DependencyTemplate for CreateScriptUrlDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CreateScriptUrlDependency>()
      .expect("CreateScriptUrlDependencyTemplate should be used for CreateScriptUrlDependency");

    code_generatable_context
      .runtime_requirements
      .insert(RuntimeGlobals::CREATE_SCRIPT_URL);

    source.insert(
      dep.range_path.start,
      format!(
        "{}(",
        code_generatable_context
          .compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::CREATE_SCRIPT_URL),
      )
      .as_str(),
      None,
    );
    source.insert(dep.range_path.end, ")", None);
  }
}
