use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  block_promise, AffectType, AsContextDependency, AsModuleDependency, Dependency,
  DependencyCategory, DependencyId, DependencyRange, DependencyTemplate, DependencyType,
  RuntimeGlobals,
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

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::False
  }
}

impl AsModuleDependency for RequireEnsureDependency {}

#[cacheable_dyn]
impl DependencyTemplate for RequireEnsureDependency {
  fn apply(
    &self,
    source: &mut rspack_core::TemplateReplaceSource,
    code_generatable_context: &mut rspack_core::TemplateContext,
  ) {
    let module_graph = code_generatable_context.compilation.get_module_graph();
    let block = module_graph.get_parent_block(&self.id);
    let promise = block_promise(
      block,
      code_generatable_context.runtime_requirements,
      code_generatable_context.compilation,
      self.dependency_type().as_str(),
    );
    source.replace(
      self.range.start,
      self.content_range.start,
      &format!("{}.then((", promise),
      None,
    );
    code_generatable_context
      .runtime_requirements
      .insert(RuntimeGlobals::REQUIRE);
    if let Some(error_handler_range) = &self.error_handler_range {
      source.replace(
        self.content_range.end,
        error_handler_range.start,
        &format!(").bind(null, {}))['catch'](", RuntimeGlobals::REQUIRE),
        None,
      );
      source.replace(error_handler_range.end, self.range.end, ")", None);
    } else {
      code_generatable_context
        .runtime_requirements
        .insert(RuntimeGlobals::UNCAUGHT_ERROR_HANDLER);
      source.replace(
        self.content_range.end,
        self.range.end,
        &format!(
          ").bind(null, {}))['catch']({})",
          RuntimeGlobals::REQUIRE,
          RuntimeGlobals::UNCAUGHT_ERROR_HANDLER
        ),
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
    _compilation: &rspack_core::Compilation,
    _runtime: Option<&rspack_core::RuntimeSpec>,
  ) {
  }
}

impl AsContextDependency for RequireEnsureDependency {}
