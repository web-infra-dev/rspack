use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  Compilation, DependencyCodeGeneration, DependencyRange, DependencyTemplate,
  DependencyTemplateType, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};
use rspack_hash::{RspackHash, RspackHasher};

/// Renders the `exports` object of the current module for the special AMD dependency string
/// `'exports'`.
///
/// The exports argument name depends on the module: it is `exports` for CommonJS/AMD modules, while
/// a module with ESM syntax uses the runtime exports variable (`__webpack_exports__` /
/// `__rspack_exports`). Resolving it at code generation time keeps the emitted reference in sync
/// with the parameter of the module factory. Referencing it also adds the `exports` runtime
/// requirement, which makes the module factory receive the exports object.
///
/// `RuntimeRequirementsDependency` must not be used here: rendering the `exports` runtime global
/// yields the runtime-level variable name instead of the module's exports argument, and a
/// `ConstDependency` cannot resolve the module-dependent name.
#[cacheable]
#[derive(Debug)]
pub struct AMDExportsDependency {
  range: DependencyRange,
}

impl AMDExportsDependency {
  pub fn new(range: DependencyRange) -> Self {
    Self { range }
  }
}

impl RspackHash for AMDExportsDependency {
  fn hash(&self, state: &mut RspackHasher) {
    "range".hash(state);
    self.range.hash(state);
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for AMDExportsDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(AMDExportsDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut RspackHasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    RspackHash::hash(self, hasher);
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct AMDExportsDependencyTemplate;

impl AMDExportsDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("AMDExportsDependency")
  }
}

impl DependencyTemplate for AMDExportsDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<AMDExportsDependency>()
      .expect("AMDExportsDependencyTemplate should be used for AMDExportsDependency");

    let TemplateContext {
      compilation,
      module,
      runtime_template,
      ..
    } = code_generatable_context;

    let content = runtime_template.render_exports_argument(
      compilation
        .get_module_graph()
        .module_by_identifier(&module.identifier())
        .expect("should have module")
        .get_exports_argument(),
    );

    source.replace(dep.range.start, dep.range.end, content, None);
  }
}
