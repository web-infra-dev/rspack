use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsTuple2, AsVec},
};
use rspack_core::{
  AsDependency, Compilation, DependencyId, DependencyTemplate, ExternalModuleInitFragment,
  InitFragmentExt, InitFragmentStage, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};
use rspack_util::ext::DynHash;

#[cacheable]
#[derive(Debug, Clone)]
pub struct ExternalModuleDependency {
  id: DependencyId,
  module: String,
  #[with(AsVec<AsTuple2>)]
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

#[cacheable_dyn]
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

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.module.dyn_hash(hasher);
    self.import_specifier.dyn_hash(hasher);
    self.default_import.dyn_hash(hasher);
  }
}

impl AsDependency for ExternalModuleDependency {}
