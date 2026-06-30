use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  Compilation, DependencyCodeGeneration, DependencyTemplate, DependencyTemplateType,
  ExternalModuleInitFragment, InitFragmentExt, InitFragmentStage, RuntimeSpec, TemplateContext,
  TemplateReplaceSource,
};
use rspack_hash::{RspackHash, RspackHashable};

#[cacheable]
#[derive(Debug, Clone, RspackHashable)]
pub struct ExternalModuleDependency {
  module: String,
  import_specifier: Vec<(String, String)>,
  #[rspack_hash(null_if_none)]
  default_import: Option<String>,
}

impl ExternalModuleDependency {
  pub fn new(
    module: String,
    import_specifier: Vec<(String, String)>,
    default_import: Option<String>,
  ) -> Self {
    Self {
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
    hasher: &mut RspackHash,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    RspackHashable::hash(self, hasher);
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
