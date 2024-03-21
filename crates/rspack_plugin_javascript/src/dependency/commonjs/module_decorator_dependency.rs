use rspack_core::{
  AsDependency, DependencyTemplate, InitFragmentKey, InitFragmentStage, NormalInitFragment,
  RuntimeGlobals, TemplateContext, TemplateReplaceSource,
};

#[derive(Debug, Clone)]
pub struct ModuleDecoratorDependency {
  decorator: RuntimeGlobals,
}

impl ModuleDecoratorDependency {
  pub fn new(decorator: RuntimeGlobals) -> Self {
    Self { decorator }
  }
}

impl DependencyTemplate for ModuleDecoratorDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      runtime_requirements,
      init_fragments,
      compilation,
      module,
      ..
    } = code_generatable_context;

    runtime_requirements.insert(RuntimeGlobals::MODULE_LOADED);
    runtime_requirements.insert(RuntimeGlobals::MODULE_ID);
    runtime_requirements.insert(RuntimeGlobals::MODULE);
    runtime_requirements.insert(self.decorator);

    let module = compilation
      .get_module_graph()
      .module_by_identifier(&module.identifier())
      .expect("should have mgm");
    let module_argument = module.get_module_argument();

    // ref: webpack-test/cases/scope-hoisting/issue-5096 will return a `null` as module id
    let module_id = compilation
      .chunk_graph
      .get_module_id(module.identifier())
      .clone()
      .unwrap_or_default();

    init_fragments.push(Box::new(NormalInitFragment::new(
      format!(
        "/* module decorator */ {} = {}({});\n",
        module_argument,
        self.decorator.name(),
        module_argument
      ),
      InitFragmentStage::StageProvides,
      0,
      InitFragmentKey::ModuleDecorator(module_id),
      None,
    )));
  }

  fn dependency_id(&self) -> Option<rspack_core::DependencyId> {
    None
  }
}
impl AsDependency for ModuleDecoratorDependency {}
