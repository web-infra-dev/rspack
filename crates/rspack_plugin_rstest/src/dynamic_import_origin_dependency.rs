use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, DependencyCodeGeneration, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, TemplateContext,
  TemplateReplaceSource,
};
use rspack_util::json_stringify_str;

#[cacheable]
#[derive(Debug, Clone)]
pub struct RstestDynamicImportOriginDependency {
  callee_range: DependencyRange,
  args_end: u32,
  /// Whether the original `import()` call had a 2nd argument (importAttributes).
  /// When false, a `void 0` placeholder is emitted so `origin` always lands
  /// at the 3rd argument position the runtime expects.
  has_attributes: bool,
  origin_path: String,
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
#[derive(Debug, Clone, Default)]
pub struct RstestDynamicImportOriginDependencyTemplate;

impl RstestDynamicImportOriginDependencyTemplate {
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

    let import_fn = code_generatable_context
      .compilation
      .options
      .output
      .import_function_name
      .clone();

    // Replace `import` callee with the configured importFunctionName.
    source.replace(
      dep.callee_range.start,
      dep.callee_range.end,
      import_fn,
      None,
    );

    // Append the origin so the runtime can resolve relative specifiers against
    // the source module that produced the dynamic import, instead of the test
    // entry. Always emit it at the third-argument position — fill the
    // attributes slot with `void 0` when the call had only a specifier. We
    // use `void 0` rather than the identifier `undefined` because the latter
    // can be shadowed (e.g. `function f(undefined) { import(spec) }`), which
    // would silently feed the runtime a bogus `importAttributes` value.
    let tail = if dep.has_attributes {
      format!(", {}", json_stringify_str(&dep.origin_path))
    } else {
      format!(", void 0, {}", json_stringify_str(&dep.origin_path))
    };
    source.replace(dep.args_end, dep.args_end, tail, None);
  }
}
