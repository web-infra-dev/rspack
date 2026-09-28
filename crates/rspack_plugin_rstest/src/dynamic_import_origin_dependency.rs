use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, DependencyCodeGeneration, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, TemplateContext,
  TemplateReplaceSource,
};
#[cfg(allocative)]
use rspack_util::allocative;
use rspack_util::json_stringify_str;

#[cacheable]
#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct RstestDynamicImportOriginDependency {
  callee_range: DependencyRange,
  args_end: u32,
  /// Whether the original `import()` call had a 2nd argument (importAttributes).
  /// When false, a `void 0` placeholder is emitted so `origin` always lands
  /// at the 3rd argument position the runtime expects.
  has_attributes: bool,
  origin_path: String,
  import_mock: bool,
  require_mock: bool,
}

impl RstestDynamicImportOriginDependency {
  pub fn new(
    callee_range: DependencyRange,
    args_end: u32,
    has_attributes: bool,
    origin_path: String,
  ) -> Self {
    Self {
      callee_range,
      args_end,
      has_attributes,
      origin_path,
      import_mock: false,
      require_mock: false,
    }
  }

  pub fn new_mock(
    callee_range: DependencyRange,
    args_end: u32,
    origin_path: String,
    is_import: bool,
  ) -> Self {
    Self {
      callee_range,
      args_end,
      has_attributes: false,
      origin_path,
      import_mock: is_import,
      require_mock: !is_import,
    }
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for RstestDynamicImportOriginDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RstestDynamicImportOriginDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for RstestDynamicImportOriginDependency {}
impl AsContextDependency for RstestDynamicImportOriginDependency {}

#[cacheable]
#[derive(Debug, Clone)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct RstestDynamicImportOriginDependencyTemplate {
  /// Resolved callee for the rewrite — rstest's own `functionName` override
  /// or the `output.importFunctionName` fallback. Resolved once at `apply`
  /// time (with the default `import` normalized to "feature off" since
  /// native `import()` only accepts 1-2 args) and held here so each
  /// `import()` call site reuses the same string without repeating the
  /// override + fallback + default-`import` check.
  function_name: String,
}

impl RstestDynamicImportOriginDependencyTemplate {
  pub fn new(function_name: String) -> Self {
    Self { function_name }
  }

  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::RstestDynamicImportOrigin)
  }
}

impl DependencyTemplate for RstestDynamicImportOriginDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<RstestDynamicImportOriginDependency>()
      .expect(
        "RstestDynamicImportOriginDependencyTemplate can only be applied to \
         RstestDynamicImportOriginDependency",
      );

    if dep.import_mock || dep.require_mock {
      let require = code_generatable_context
        .runtime_template
        .render_runtime_globals(&rspack_core::RuntimeGlobals::REQUIRE);
      source.replace(
        dep.callee_range.start,
        dep.callee_range.end,
        format!(
          "{require}.rstest_{}",
          if dep.import_mock {
            "import_mock"
          } else {
            "require_mock"
          }
        ),
        None,
      );
      source.replace(
        dep.args_end,
        dep.args_end,
        format!(", {}", json_stringify_str(&dep.origin_path)),
        None,
      );
      return;
    }

    source.replace(
      dep.callee_range.start,
      dep.callee_range.end,
      self.function_name.clone(),
      None,
    );

    // `void 0` (rather than the identifier `undefined`) for the missing
    // attributes slot — `undefined` can be shadowed by a local binding.
    let tail = if dep.has_attributes {
      format!(", {}", json_stringify_str(&dep.origin_path))
    } else {
      format!(", void 0, {}", json_stringify_str(&dep.origin_path))
    };
    source.replace(dep.args_end, dep.args_end, tail, None);
  }
}
