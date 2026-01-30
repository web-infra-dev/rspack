use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsOption, AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType, ExportsType,
  ExtendedReferencedExport, FactorizeInfo, ImportAttributes, ModuleDependency, ModuleGraph,
  ModuleGraphCacheArtifact, ReferencedExport, ResourceIdentifier, TemplateContext,
  TemplateReplaceSource, create_exports_object_referenced,
};
use swc_core::ecma::atoms::Atom;

use super::create_resource_identifier_for_esm_dependency;

pub fn create_import_dependency_referenced_exports(
  dependency_id: DependencyId,
  referenced_exports: &Option<Vec<Vec<Atom>>>,
  mg: &ModuleGraph,
  mg_cache: &ModuleGraphCacheArtifact,
) -> Vec<ExtendedReferencedExport> {
  if let Some(referenced_exports) = referenced_exports {
    let mut refs = vec![];
    for referenced_export in referenced_exports {
      if let Some(first) = referenced_export.first()
        && first == "default"
      {
        let Some(strict) = mg
          .get_parent_module(&dependency_id)
          .and_then(|id| mg.module_by_identifier(id))
          .map(|m| m.build_meta().strict_esm_module)
        else {
          return create_exports_object_referenced();
        };
        let Some(imported_module) = mg
          .module_identifier_by_dependency_id(&dependency_id)
          .and_then(|id| mg.module_by_identifier(id))
        else {
          return create_exports_object_referenced();
        };
        let exports_type = imported_module.get_exports_type(mg, mg_cache, strict);
        if matches!(
          exports_type,
          ExportsType::DefaultOnly | ExportsType::DefaultWithNamed
        ) {
          return create_exports_object_referenced();
        }
      }
      refs.push(ExtendedReferencedExport::Export(ReferencedExport::new(
        referenced_export.clone(),
        false,
        false,
      )));
    }
    refs
  } else {
    create_exports_object_referenced()
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct ImportDependency {
  pub id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  pub range: DependencyRange,
  #[cacheable(with=AsOption<AsVec<AsVec<AsPreset>>>)]
  referenced_exports: Option<Vec<Vec<Atom>>>,
  attributes: Option<ImportAttributes>,
  pub comments: Vec<(bool, String)>,
  resource_identifier: ResourceIdentifier,
  factorize_info: FactorizeInfo,
  optional: bool,
}

impl ImportDependency {
  pub fn new(
    request: Atom,
    range: DependencyRange,
    referenced_exports: Option<Vec<Vec<Atom>>>,
    attributes: Option<ImportAttributes>,
    optional: bool,
    comments: Vec<(bool, String)>,
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
      optional,
      comments,
    }
  }

  pub fn set_referenced_exports(&mut self, referenced_exports: Vec<Vec<Atom>>) {
    self.referenced_exports = Some(referenced_exports);
  }
}

#[cacheable_dyn]
impl Dependency for ImportDependency {
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
    &DependencyType::DynamicImport
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
      self.id,
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
impl ModuleDependency for ImportDependency {
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

  fn get_optional(&self) -> bool {
    self.optional
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ImportDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportDependencyTemplate::template_type())
  }
}

impl AsContextDependency for ImportDependency {}

#[cacheable]
#[derive(Debug, Default)]
pub struct ImportDependencyTemplate;

impl ImportDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::DynamicImport)
  }
}

impl DependencyTemplate for ImportDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportDependency>()
      .expect("ImportDependencyTemplate can only be applied to ImportDependency");
    let range = dep.range().expect("ImportDependency should have range");
    let module_graph = code_generatable_context.compilation.get_module_graph();
    let block = module_graph.get_parent_block(dep.id());
    source.replace(
      range.start,
      range.end,
      code_generatable_context
        .runtime_template
        .module_namespace_promise(
          code_generatable_context.compilation,
          code_generatable_context.module.identifier(),
          dep.id(),
          block,
          dep.request(),
          dep.dependency_type().as_str(),
          false,
        )
        .as_str(),
      None,
    );
  }
}
