use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsOption, AsPreset, AsVec},
};
use rspack_core::{
  module_namespace_promise, AsContextDependency, Compilation, Dependency, DependencyCategory,
  DependencyId, DependencyRange, DependencyTemplate, DependencyType, FactorizeInfo,
  ImportAttributes, ModuleDependency, RuntimeSpec, TemplateContext, TemplateReplaceSource,
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
  #[cacheable(with=AsOption<AsVec<AsPreset>>)]
  referenced_exports: Option<Vec<Atom>>,
  attributes: Option<ImportAttributes>,
  resource_identifier: String,
  factorize_info: FactorizeInfo,
}

impl ImportEagerDependency {
  pub fn new(
    request: Atom,
    range: DependencyRange,
    referenced_exports: Option<Vec<Atom>>,
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

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
  }

  fn get_referenced_exports(
    &self,
    module_graph: &rspack_core::ModuleGraph,
    _runtime: Option<&rspack_core::RuntimeSpec>,
  ) -> Vec<rspack_core::ExtendedReferencedExport> {
    create_import_dependency_referenced_exports(&self.id, &self.referenced_exports, module_graph)
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

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyTemplate for ImportEagerDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let module_graph = code_generatable_context.compilation.get_module_graph();
    let block = module_graph.get_parent_block(&self.id);
    source.replace(
      self.range.start,
      self.range.end,
      module_namespace_promise(
        code_generatable_context,
        &self.id,
        block,
        &self.request,
        self.dependency_type().as_str(),
        false,
      )
      .as_str(),
      None,
    );
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }
}

impl AsContextDependency for ImportEagerDependency {}
