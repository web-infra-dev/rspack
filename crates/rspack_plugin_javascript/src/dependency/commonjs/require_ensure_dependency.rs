use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AffectType, AsContextDependency, AsModuleDependency, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, RuntimeGlobals, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireEnsureDependency {
  id: DependencyId,
  range: DependencyRange,
  content_range: DependencyRange,
  error_handler_range: Option<DependencyRange>,
}

impl RequireEnsureDependency {
  pub fn new(
    range: DependencyRange,
    content_range: DependencyRange,
    error_handler_range: Option<DependencyRange>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      content_range,
      error_handler_range,
    }
  }
}

#[cacheable_dyn]
impl Dependency for RequireEnsureDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RequireEnsure
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::False
  }
}

impl AsModuleDependency for RequireEnsureDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for RequireEnsureDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RequireEnsureDependencyTemplate::template_type())
  }
}

impl AsContextDependency for RequireEnsureDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RequireEnsureDependencyTemplate;

impl RequireEnsureDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::RequireEnsure)
  }
}

impl DependencyTemplate for RequireEnsureDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<RequireEnsureDependency>()
      .expect("RequireEnsureDependencyTemplate should be used for RequireEnsureDependency");

    let module_graph = code_generatable_context.compilation.get_module_graph();
    let block = module_graph.get_parent_block(&dep.id);
    let promise = code_generatable_context
      .compilation
      .runtime_template
      .block_promise(
        block,
        code_generatable_context.runtime_requirements,
        code_generatable_context.compilation,
        dep.dependency_type().as_str(),
      );
    source.replace(
      dep.range.start,
      dep.content_range.start,
      &format!("{promise}.then(("),
      None,
    );
    code_generatable_context
      .runtime_requirements
      .insert(RuntimeGlobals::REQUIRE);
    if let Some(error_handler_range) = &dep.error_handler_range {
      source.replace(
        dep.content_range.end,
        error_handler_range.start,
        &format!(
          ").bind(null, {}))['catch'](",
          code_generatable_context
            .compilation
            .runtime_template
            .render_runtime_globals(&RuntimeGlobals::REQUIRE)
        ),
        None,
      );
      source.replace(error_handler_range.end, dep.range.end, ")", None);
    } else {
      code_generatable_context
        .runtime_requirements
        .insert(RuntimeGlobals::UNCAUGHT_ERROR_HANDLER);
      source.replace(
        dep.content_range.end,
        dep.range.end,
        &format!(
          ").bind(null, {}))['catch']({})",
          code_generatable_context
            .compilation
            .runtime_template
            .render_runtime_globals(&RuntimeGlobals::REQUIRE),
          code_generatable_context
            .compilation
            .runtime_template
            .render_runtime_globals(&RuntimeGlobals::UNCAUGHT_ERROR_HANDLER)
        ),
        None,
      );
    }
  }
}
