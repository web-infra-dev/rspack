use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsOption, AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType, ExportsInfoArtifact,
  ExportsType, ExtendedReferencedExport, FactorizeInfo, ImportAttributes, ImportPhase,
  ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact, ReferencedExport, ReferencedSpecifier,
  ResourceIdentifier, TemplateContext, TemplateReplaceSource, create_exports_object_referenced,
  get_exports_type,
};
use swc_core::ecma::atoms::Atom;

use super::create_resource_identifier_for_esm_dependency;

pub fn create_import_dependency_referenced_exports(
  dependency_id: &DependencyId,
  referenced_specifiers: &Option<Vec<ReferencedSpecifier>>,
  mg: &ModuleGraph,
  mg_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Vec<ExtendedReferencedExport> {
  if let Some(referenced_specifiers) = referenced_specifiers {
    let mut refs = vec![];
    for ReferencedSpecifier {
      names,
      is_call,
      namespace_object_as_context,
    } in referenced_specifiers
    {
      let mut names = names.as_slice();
      let mut namespace_object_as_context = *namespace_object_as_context;
      let parent_module = mg
        .get_parent_module(dependency_id)
        .expect("should have parent module");
      let exports_type = get_exports_type(
        mg,
        mg_cache,
        exports_info_artifact,
        dependency_id,
        parent_module,
      );

      // Force enable namespace object as context for DefaultOnly and DefaultWithNamed
      // because it's more common in cjs and json
      if matches!(
        exports_type,
        ExportsType::DefaultOnly | ExportsType::DefaultWithNamed
      ) {
        namespace_object_as_context = true;
      }

      if let Some(id) = names.first()
        && id == "default"
      {
        match exports_type {
          ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
            if names.len() == 1 {
              return create_exports_object_referenced();
            }
            names = &names[1..];
          }
          ExportsType::Dynamic => {
            return create_exports_object_referenced();
          }
          _ => {}
        }
      }

      if namespace_object_as_context && *is_call {
        if names.len() == 1 {
          return create_exports_object_referenced();
        }
        // remove last one
        names = &names[..names.len().saturating_sub(1)];
      }
      refs.push(ExtendedReferencedExport::Export(ReferencedExport::new(
        names.to_vec(),
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
  #[cacheable(with=AsOption<AsVec<AsCacheable>>)]
  referenced_specifiers: Option<Vec<ReferencedSpecifier>>,
  attributes: Option<ImportAttributes>,
  phase: ImportPhase,
  pub comments: Vec<(bool, String)>,
  resource_identifier: ResourceIdentifier,
  factorize_info: FactorizeInfo,
  optional: bool,
}

impl ImportDependency {
  pub fn new(
    request: Atom,
    range: DependencyRange,
    referenced_specifiers: Option<Vec<ReferencedSpecifier>>,
    attributes: Option<ImportAttributes>,
    phase: ImportPhase,
    optional: bool,
    comments: Vec<(bool, String)>,
  ) -> Self {
    let resource_identifier =
      create_resource_identifier_for_esm_dependency(request.as_str(), attributes.as_ref());
    Self {
      request,
      range,
      id: DependencyId::new(),
      referenced_specifiers,
      attributes,
      phase,
      resource_identifier,
      factorize_info: Default::default(),
      optional,
      comments,
    }
  }

  pub fn set_referenced_specifiers(&mut self, referenced_specifiers: Vec<ReferencedSpecifier>) {
    self.referenced_specifiers = Some(referenced_specifiers);
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

  fn get_phase(&self) -> ImportPhase {
    self.phase
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn get_referenced_exports(
    &self,
    module_graph: &rspack_core::ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&rspack_core::RuntimeSpec>,
  ) -> Vec<rspack_core::ExtendedReferencedExport> {
    create_import_dependency_referenced_exports(
      &self.id,
      &self.referenced_specifiers,
      module_graph,
      module_graph_cache,
      exports_info_artifact,
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
          dep.get_phase(),
        ),
      None,
    );
  }
}
