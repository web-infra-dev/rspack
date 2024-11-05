use rspack_core::{
  block_promise, AffectType, AsContextDependency, AsModuleDependency, Compilation, Dependency,
  DependencyCategory, DependencyId, DependencyTemplate, DependencyType, RuntimeGlobals,
  RuntimeSpec,
};

#[derive(Debug, Clone)]
pub struct AMDRequireDependency {
  id: DependencyId,
  outer_range: (u32, u32),
  // In the webpack source code, type annotation of `arrayRange` is non-null.
  // However, `DependencyTemplate` implementation assumes `arrayRange` can be null in some cases.
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

impl DependencyTemplate for AMDRequireDependency {
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
      "AMD require",
    );

    // has array range but no function range
    if self.array_range.is_some() && self.function_range.is_none() {
      let start_block = promise + ".then(function() {";
      let end_block = format!(
        ";}})['catch']{}",
        RuntimeGlobals::UNCAUGHT_ERROR_HANDLER.name()
      );
      code_generatable_context
        .runtime_requirements
        .insert(RuntimeGlobals::UNCAUGHT_ERROR_HANDLER);
      let array_range = self.array_range.unwrap();
      source.replace(self.outer_range.0, array_range.0, &start_block, None);
      source.replace(array_range.1, self.outer_range.1, &end_block, None);
      return;
    }

    // has function range but no array range
    if self.function_range.is_some() && self.array_range.is_none() {
      let start_block = promise + ".then((";
      let end_block = format!(
        ").bind(exports, {}, exports, module))['catch']({})",
        RuntimeGlobals::REQUIRE.name(),
        RuntimeGlobals::UNCAUGHT_ERROR_HANDLER.name()
      );
      code_generatable_context
        .runtime_requirements
        .insert(RuntimeGlobals::UNCAUGHT_ERROR_HANDLER);
      let function_range = self.function_range.unwrap();
      source.replace(self.outer_range.0, function_range.0, &start_block, None);
      source.replace(function_range.1, self.outer_range.1, &end_block, None);
      return;
    }

    // has array range, function range, and errorCallbackRange
    if self.array_range.is_some()
      && self.function_range.is_some()
      && self.error_callback_range.is_some()
    {
      let start_block = promise + ".then(function() { ";
      let error_range_block = if self.function_bind_this {
        "}.bind(this))['catch']("
      } else {
        "})['catch']("
      };
      let end_block = if self.error_callback_bind_this {
        ".bind(this))"
      } else {
        ")"
      };
      let array_range = self.array_range.unwrap();
      let function_range = self.function_range.unwrap();
      let error_callback_range = self.error_callback_range.unwrap();

      source.replace(self.outer_range.0, array_range.0, &start_block, None);

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

      source.replace(error_callback_range.1, self.outer_range.1, end_block, None);

      return;
    }

    // has array range, function range, but no errorCallbackRange
    if self.array_range.is_some() && self.function_range.is_some() {
      let start_block = promise + ".then(function() { ";
      let end_block = format!(
        "}}{})['catch']({})",
        if self.function_bind_this {
          ".bind(this)"
        } else {
          ""
        },
        RuntimeGlobals::UNCAUGHT_ERROR_HANDLER.name()
      );
      code_generatable_context
        .runtime_requirements
        .insert(RuntimeGlobals::UNCAUGHT_ERROR_HANDLER);

      let array_range = self.array_range.unwrap();
      let function_range = self.function_range.unwrap();

      source.replace(self.outer_range.0, array_range.0, &start_block, None);

      source.insert(array_range.0, "var __WEBPACK_AMD_REQUIRE_ARRAY__ = ", None);

      source.replace(array_range.1, function_range.0, "; (", None);

      source.insert(
        function_range.1,
        ").apply(null, __WEBPACK_AMD_REQUIRE_ARRAY__);",
        None,
      );

      source.replace(function_range.1, self.outer_range.1, &end_block, None);
    };
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

impl AsModuleDependency for AMDRequireDependency {}

impl AsContextDependency for AMDRequireDependency {}
