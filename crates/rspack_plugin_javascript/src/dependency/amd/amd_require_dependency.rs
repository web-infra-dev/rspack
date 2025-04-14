use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  block_promise, AffectType, AsContextDependency, AsModuleDependency, Dependency,
  DependencyCategory, DependencyCodeGeneration, DependencyId, DependencyTemplate,
  DependencyTemplateType, DependencyType, RuntimeGlobals, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct AMDRequireDependency {
  id: DependencyId,
  outer_range: (u32, u32),
  // In the webpack source code, type annotation of `arrayRange` is non-null.
  // However, `DependencyCodeGeneration` implementation assumes `arrayRange` can be null in some cases.
  // So I use Option here.
  array_range: Option<(u32, u32)>,
  function_range: Option<(u32, u32)>,
  error_callback_range: Option<(u32, u32)>,
  pub function_bind_this: bool,
  pub error_callback_bind_this: bool,
}

impl AMDRequireDependency {
  pub fn new(
    outer_range: (u32, u32),
    array_range: Option<(u32, u32)>,
    function_range: Option<(u32, u32)>,
    error_callback_range: Option<(u32, u32)>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      outer_range,
      array_range,
      function_range,
      error_callback_range,
      function_bind_this: false,
      error_callback_bind_this: false,
    }
  }
}

#[cacheable_dyn]
impl Dependency for AMDRequireDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Amd
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::AmdRequire
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::False
  }
}

impl AsModuleDependency for AMDRequireDependency {}

impl AsContextDependency for AMDRequireDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for AMDRequireDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(AMDRequireDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct AMDRequireDependencyTemplate;

impl AMDRequireDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::AmdRequire)
  }
}

impl DependencyTemplate for AMDRequireDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<AMDRequireDependency>()
      .expect("AMDRequireDependencyTemplate should only be used for AMDRequireDependency");

    let module_graph = code_generatable_context.compilation.get_module_graph();
    let block = module_graph.get_parent_block(&dep.id);

    let promise = block_promise(
      block,
      code_generatable_context.runtime_requirements,
      code_generatable_context.compilation,
      "AMD require",
    );

    // has array range but no function range
    if let Some(array_range) = dep.array_range
      && dep.function_range.is_none()
    {
      let start_block = promise + ".then(function() {";
      let end_block = format!(
        ";}})['catch']({})",
        RuntimeGlobals::UNCAUGHT_ERROR_HANDLER.name()
      );
      code_generatable_context
        .runtime_requirements
        .insert(RuntimeGlobals::UNCAUGHT_ERROR_HANDLER);
      source.replace(dep.outer_range.0, array_range.0, &start_block, None);
      source.replace(array_range.1, dep.outer_range.1, &end_block, None);
      return;
    }

    // has function range but no array range
    if let Some(function_range) = dep.function_range
      && dep.array_range.is_none()
    {
      let start_block = promise + ".then((";
      let end_block = format!(
        ").bind(exports, {}, exports, module))['catch']({})",
        RuntimeGlobals::REQUIRE.name(),
        RuntimeGlobals::UNCAUGHT_ERROR_HANDLER.name()
      );
      code_generatable_context
        .runtime_requirements
        .insert(RuntimeGlobals::UNCAUGHT_ERROR_HANDLER);
      source.replace(dep.outer_range.0, function_range.0, &start_block, None);
      source.replace(function_range.1, dep.outer_range.1, &end_block, None);
      return;
    }

    // has array range, function range, and errorCallbackRange
    if let Some(array_range) = dep.array_range
      && let Some(function_range) = dep.function_range
      && let Some(error_callback_range) = dep.error_callback_range
    {
      let start_block = promise + ".then(function() { ";
      let error_range_block = if dep.function_bind_this {
        "}.bind(this))['catch']("
      } else {
        "})['catch']("
      };
      let end_block = if dep.error_callback_bind_this {
        ".bind(this))"
      } else {
        ")"
      };

      source.replace(dep.outer_range.0, array_range.0, &start_block, None);

      source.insert(array_range.0, "var __WEBPACK_AMD_REQUIRE_ARRAY__ = ", None);

      source.replace(array_range.1, function_range.0, "; (", None);

      source.insert(
        function_range.1,
        ").apply(null, __WEBPACK_AMD_REQUIRE_ARRAY__);",
        None,
      );

      source.replace(
        function_range.1,
        error_callback_range.0,
        error_range_block,
        None,
      );

      source.replace(error_callback_range.1, dep.outer_range.1, end_block, None);

      return;
    }

    // has array range, function range, but no errorCallbackRange
    if let Some(array_range) = dep.array_range
      && let Some(function_range) = dep.function_range
    {
      let start_block = promise + ".then(function() { ";
      let end_block = format!(
        "}}{})['catch']({})",
        if dep.function_bind_this {
          ".bind(this)"
        } else {
          ""
        },
        RuntimeGlobals::UNCAUGHT_ERROR_HANDLER.name()
      );
      code_generatable_context
        .runtime_requirements
        .insert(RuntimeGlobals::UNCAUGHT_ERROR_HANDLER);

      source.replace(dep.outer_range.0, array_range.0, &start_block, None);

      source.insert(array_range.0, "var __WEBPACK_AMD_REQUIRE_ARRAY__ = ", None);

      source.replace(array_range.1, function_range.0, "; (", None);

      source.insert(
        function_range.1,
        ").apply(null, __WEBPACK_AMD_REQUIRE_ARRAY__);",
        None,
      );

      source.replace(function_range.1, dep.outer_range.1, &end_block, None);
    };
  }
}
