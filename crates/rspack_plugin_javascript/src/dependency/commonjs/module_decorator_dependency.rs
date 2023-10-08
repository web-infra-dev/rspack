use rspack_core::{
  DependencyTemplate, InitFragmentKey, InitFragmentStage, NormalInitFragment, RuntimeGlobals,
  TemplateContext, TemplateReplaceSource,
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

    let mgm = compilation
      .module_graph
      .module_graph_module_by_identifier(&module.identifier())
      .expect("should have mgm");
    let module_argument = mgm.get_module_argument();

    let module_id = compilation
      .chunk_graph
      .get_module_id(module.identifier())
      .clone()
      .expect("should have module_id in <ModuleDecoratorDependency as DependencyTemplate>::apply");

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
}
