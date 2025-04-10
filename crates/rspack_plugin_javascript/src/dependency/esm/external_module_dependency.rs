use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  Compilation, DependencyCodeGeneration, DependencyId, DependencyTemplate, DependencyTemplateType,
  ExternalModuleInitFragment, InitFragmentExt, InitFragmentStage, RuntimeSpec, TemplateContext,
  TemplateReplaceSource,
};
use rspack_util::ext::DynHash;

#[cacheable]
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

#[cacheable_dyn]
impl DependencyCodeGeneration for ExternalModuleDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ExternalModuleDependencyTemplate::template_type())
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

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ExternalModuleDependencyTemplate;

impl ExternalModuleDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ExternalModuleDependency")
  }
}

impl DependencyTemplate for ExternalModuleDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ExternalModuleDependency>()
      .expect("ExternalModuleDependencyTemplate should only be used for ExternalModuleDependency");
    let need_prefix = code_generatable_context
      .compilation
      .options
      .output
      .environment
      .supports_node_prefix_for_core_modules();
    let chunk_init_fragments = code_generatable_context.chunk_init_fragments();
    let fragment = ExternalModuleInitFragment::new(
      format!("{}{}", if need_prefix { "node:" } else { "" }, dep.module),
      dep.import_specifier.clone(),
      dep.default_import.clone(),
      InitFragmentStage::StageConstants,
      0,
    );
    chunk_init_fragments.push(fragment.boxed());
  }
}
