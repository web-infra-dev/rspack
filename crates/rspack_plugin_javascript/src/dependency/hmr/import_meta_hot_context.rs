use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  Compilation, DependencyCodeGeneration, DependencyLocation, DependencyRange, DependencyTemplate,
  DependencyTemplateType, RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};
use rspack_hash::{RspackHash, RspackHasher};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ImportMetaHotDependency {
  range: DependencyRange,
  loc: Option<DependencyLocation>,
}

impl ImportMetaHotDependency {
  pub fn new(range: DependencyRange, loc: Option<DependencyLocation>) -> Self {
    Self { range, loc }
  }

  pub fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }
}

impl RspackHash for ImportMetaHotDependency {
  fn hash(&self, state: &mut RspackHasher) {
    "ImportMetaHotDependency".hash(state);
    self.range.hash(state);
    RuntimeGlobals::HOT_CONTEXT.hash(state);
    RuntimeGlobals::MODULE.hash(state);
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ImportMetaHotDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportMetaHotDependencyTemplate::template_type())
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
pub struct ImportMetaHotDependencyTemplate;

impl ImportMetaHotDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ImportMetaHotDependency")
  }
}

impl DependencyTemplate for ImportMetaHotDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportMetaHotDependency>()
      .expect("ImportMetaHotDependencyTemplate requires ImportMetaHotDependency");
    let module_argument = context.runtime_template.render_module_argument(
      context
        .compilation
        .get_module_graph()
        .module_by_identifier(&context.module.identifier())
        .expect("module graph module must exist")
        .get_module_argument(),
    );
    context
      .runtime_template
      .runtime_requirements_mut()
      .insert(RuntimeGlobals::HOT_CONTEXT | RuntimeGlobals::MODULE);
    let getter = context
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::HOT_CONTEXT);
    source.replace(
      dep.range.start,
      dep.range.end,
      format!("{getter}({module_argument}.hot)"),
      None,
    );
  }
}
