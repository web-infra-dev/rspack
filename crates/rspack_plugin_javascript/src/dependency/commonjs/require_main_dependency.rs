use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  Compilation, DependencyCodeGeneration, DependencyRange, DependencyTemplate,
  DependencyTemplateType, RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};
use rspack_util::ext::DynHash;

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireMainDependency {
  pub range: DependencyRange,
}

impl RequireMainDependency {
  pub fn new(range: DependencyRange) -> Self {
    Self { range }
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for RequireMainDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RequireMainDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.range.dyn_hash(hasher);
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RequireMainDependencyTemplate;

impl RequireMainDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("RequireMainDependency")
  }
}

impl DependencyTemplate for RequireMainDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<RequireMainDependency>()
      .expect("RequireMainDependencyTemplate should be used for RequireMainDependency");
    let content = format!(
      "{}[{}]",
      code_generatable_context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::MODULE_CACHE),
      code_generatable_context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::ENTRY_MODULE_ID)
    );
    source.replace(dep.range.start, dep.range.end, &content, None);
  }
}
