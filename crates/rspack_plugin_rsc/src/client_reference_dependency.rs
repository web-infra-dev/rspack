use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, Compilation, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyId, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  ExtendedReferencedExport, FactorizeInfo, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ClientReferenceDependency {
  id: DependencyId,
  request: String,
  factorize_info: FactorizeInfo,
}

impl ClientReferenceDependency {
  pub fn new(request: String) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for ClientReferenceDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::ClientReference
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::NewWorker
  }

  fn range(&self) -> Option<DependencyRange> {
    None
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    vec![]
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for ClientReferenceDependency {
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
impl DependencyCodeGeneration for ClientReferenceDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ClientReferenceDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }
}

impl AsContextDependency for ClientReferenceDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ClientReferenceDependencyTemplate;

impl ClientReferenceDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::NewWorker)
  }
}

impl DependencyTemplate for ClientReferenceDependencyTemplate {
  fn render(
    &self,
    _dep: &dyn DependencyCodeGeneration,
    _source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
  }
}
