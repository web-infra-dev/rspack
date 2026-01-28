use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  Compilation, DependencyCodeGeneration, DependencyLocation, DependencyRange, DependencyTemplate,
  DependencyTemplateType, RuntimeGlobals, RuntimeSpec, TemplateContext,
  TemplateReplaceSource,
};
use rspack_util::ext::DynHash;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ModuleArgumentDependency {
  id: Option<String>,
  range: DependencyRange,
  loc: Option<DependencyLocation>,
}

impl ModuleArgumentDependency {
  pub fn new(
    id: Option<String>,
    range: DependencyRange,
    source: Option<&str>,
  ) -> Self {
    let loc = range.to_loc(source);
    Self { id, range, loc }
  }

  pub fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ModuleArgumentDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ModuleArgumentDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.id.dyn_hash(hasher);
    self.range.dyn_hash(hasher);
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ModuleArgumentDependencyTemplate;

impl ModuleArgumentDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ModuleArgumentDependency")
  }
}

impl DependencyTemplate for ModuleArgumentDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ModuleArgumentDependency>()
      .expect("ModuleArgumentDependencyTemplate should be used for ModuleArgumentDependency");

    let TemplateContext {
      runtime_requirements,
      compilation,
      module,
      ..
    } = code_generatable_context;

    runtime_requirements.insert(RuntimeGlobals::MODULE);

    let module_argument = compilation.runtime_template.render_module_argument(
      compilation
        .get_module_graph()
        .module_by_identifier(&module.identifier())
        .expect("should have mgm")
        .get_module_argument(),
    );

    let content = if let Some(id) = &dep.id {
      format!("{module_argument}.{id}")
    } else {
      module_argument
    };

    source.replace(dep.range.start, dep.range.end, content.as_str(), None);
  }
}
