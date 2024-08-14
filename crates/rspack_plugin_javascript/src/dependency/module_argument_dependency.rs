use rspack_core::{
  AsDependency, Compilation, DependencyTemplate, ErrorSpan, RuntimeGlobals, RuntimeSpec,
  TemplateContext, TemplateReplaceSource,
};
use rspack_error::ErrorLocation;
use rspack_util::ext::DynHash;

#[derive(Debug, Clone)]
pub struct ModuleArgumentDependency {
  id: Option<&'static str>,
  loc: ErrorLocation,
  span: ErrorSpan,
}

impl ModuleArgumentDependency {
  pub fn new(id: Option<&'static str>, loc: ErrorLocation, span: ErrorSpan) -> Self {
    Self { id, loc, span }
  }

  pub fn loc(&self) -> Option<ErrorLocation> {
    Some(self.loc)
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

    let content = if let Some(id) = self.id {
      format!("{module_argument}.{id}")
    } else {
      format!("{module_argument}")
    };

    source.replace(self.span.start, self.span.end, content.as_str(), None);
  }

  fn dependency_id(&self) -> Option<rspack_core::DependencyId> {
    None
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.id.dyn_hash(hasher);
    self.span.dyn_hash(hasher);
  }
}

impl AsDependency for ModuleArgumentDependency {}
