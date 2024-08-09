use rspack_core::{
  AsDependency, DependencyId, DependencyTemplate, ExternalModuleInitFragment, InitFragmentExt,
  InitFragmentStage, TemplateContext, TemplateReplaceSource,
};

#[derive(Debug, Clone)]
pub struct ExternalModuleDependency {
  id: DependencyId,
  module: String,
  import_specifier: Vec<(String, String)>,
  default_import: Option<String>,
}

impl ExternalModuleDependency {
  pub fn new(
    module: String,
    import_specifier: Vec<(String, String)>,
    default_import: Option<String>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      module,
      import_specifier,
      default_import,
    }
  }
}

impl DependencyTemplate for ExternalModuleDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let chunk_init_fragments = code_generatable_context.chunk_init_fragments();
    let fragment = ExternalModuleInitFragment::new(
      self.module.clone(),
      self.import_specifier.clone(),
      self.default_import.clone(),
      InitFragmentStage::StageConstants,
      0,
    );
    chunk_init_fragments.push(fragment.boxed());
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsDependency for ExternalModuleDependency {}
