use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsOption, AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType, FactorizeInfo,
  ImportAttributes, ModuleDependency, ModuleGraphCacheArtifact, ResourceIdentifier,
  TemplateContext, TemplateReplaceSource,
};
use swc_core::ecma::atoms::Atom;

use super::{
  create_resource_identifier_for_esm_dependency,
  import_dependency::create_import_dependency_referenced_exports,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ImportEagerDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  range: DependencyRange,
  #[cacheable(with=AsOption<AsVec<AsVec<AsPreset>>>)]
  referenced_exports: Option<Vec<Vec<Atom>>>,
  attributes: Option<ImportAttributes>,
  resource_identifier: ResourceIdentifier,
  factorize_info: FactorizeInfo,
}

impl ImportEagerDependency {
  pub fn new(
    request: Atom,
    range: DependencyRange,
    referenced_exports: Option<Vec<Vec<Atom>>>,
    attributes: Option<ImportAttributes>,
  ) -> Self {
    let resource_identifier =
      create_resource_identifier_for_esm_dependency(request.as_str(), attributes.as_ref());
    Self {
      request,
      range,
      id: DependencyId::new(),
      referenced_exports,
      attributes,
      resource_identifier,
      factorize_info: Default::default(),
    }
  }

  pub fn set_referenced_exports(&mut self, referenced_exports: Vec<Vec<Atom>>) {
    self.referenced_exports = Some(referenced_exports);
  }
}

#[cacheable_dyn]
impl Dependency for ImportEagerDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::DynamicImportEager
  }

  fn get_attributes(&self) -> Option<&ImportAttributes> {
    self.attributes.as_ref()
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn get_referenced_exports(
    &self,
    module_graph: &rspack_core::ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    _runtime: Option<&rspack_core::RuntimeSpec>,
  ) -> Vec<rspack_core::ExtendedReferencedExport> {
    create_import_dependency_referenced_exports(
      &self.id,
      &self.referenced_exports,
      module_graph,
      module_graph_cache,
    )
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ImportEagerDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ImportEagerDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportEagerDependencyTemplate::template_type())
  }
}

impl AsContextDependency for ImportEagerDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ImportEagerDependencyTemplate;

impl ImportEagerDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ImportEagerDependency")
  }
}

impl DependencyTemplate for ImportEagerDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportEagerDependency>()
      .expect("ImportEagerDependencyTemplate should only be used for ImportEagerDependency");

    let module_graph = code_generatable_context.compilation.get_module_graph();
    let block = module_graph.get_parent_block(&dep.id);
    source.replace(
      dep.range.start,
      dep.range.end,
      code_generatable_context
        .compilation
        .runtime_template
        .module_namespace_promise(
          code_generatable_context,
          &dep.id,
          block,
          &dep.request,
          dep.dependency_type().as_str(),
          false,
        )
        .as_str(),
      None,
    );
  }
}
