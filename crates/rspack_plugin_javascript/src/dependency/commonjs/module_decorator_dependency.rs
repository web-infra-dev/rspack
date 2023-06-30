use rspack_core::{
  CodeGeneratableContext, CodeGeneratableDependency, CodeGeneratableSource, InitFragment,
  InitFragmentStage, RuntimeGlobals,
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

impl CodeGeneratableDependency for ModuleDecoratorDependency {
  fn apply(
    &self,
    _source: &mut CodeGeneratableSource,
    code_generatable_context: &mut CodeGeneratableContext,
  ) {
    let CodeGeneratableContext {
      runtime_requirements,
      init_fragments,
      compilation,
      module,
      ..
    } = code_generatable_context;

    runtime_requirements.add(RuntimeGlobals::MODULE_LOADED);
    runtime_requirements.add(RuntimeGlobals::MODULE_ID);
    runtime_requirements.add(RuntimeGlobals::MODULE);
    runtime_requirements.add(self.decorator);

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
      InitFragmentStage::STAGE_PROVIDES,
      None,
    ));
  }
}
