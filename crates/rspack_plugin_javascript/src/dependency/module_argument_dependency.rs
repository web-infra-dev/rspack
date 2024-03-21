use rspack_core::{
  AsDependency, DependencyTemplate, RuntimeGlobals, TemplateContext, TemplateReplaceSource,
};

#[derive(Debug, Clone)]
pub struct ModuleArgumentDependency {
  pub start: u32,
  pub end: u32,
  pub id: Option<&'static str>,
}

impl ModuleArgumentDependency {
  pub fn new(start: u32, end: u32, id: Option<&'static str>) -> Self {
    Self { start, end, id }
  }
}

impl DependencyTemplate for ModuleArgumentDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      runtime_requirements,
      compilation,
      module,
      ..
    } = code_generatable_context;

    runtime_requirements.insert(RuntimeGlobals::MODULE);

    let module_argument = compilation
      .get_module_graph()
      .module_by_identifier(&module.identifier())
      .expect("should have mgm")
      .get_module_argument();

    if let Some(id) = self.id {
      source.replace(
        self.start,
        self.end,
        format!("{module_argument}.{id}").as_str(),
        None,
      );
    } else {
      source.replace(self.start, self.end, &format!("{module_argument}"), None);
    }
  }

  fn dependency_id(&self) -> Option<rspack_core::DependencyId> {
    None
  }
}

impl AsDependency for ModuleArgumentDependency {}
