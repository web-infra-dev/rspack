use rspack_core::{
  DependencyTemplate, InitFragment, InitFragmentStage, RuntimeGlobals, TemplateContext,
  TemplateReplaceSource,
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

    let module_argument = compilation
      .module_graph
      .module_graph_module_by_identifier(&module.identifier())
      .expect("should have mgm")
      .get_module_argument();

    init_fragments.push(InitFragment::new(
      format!(
        "/* module decorator */ {} = {}({});\n",
        module_argument,
        self.decorator.name(),
        module_argument
      ),
      InitFragmentStage::StageProvides,
      None,
    ));
  }
}
