use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, ConditionalInitFragment, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExtendedReferencedExport, FactorizeInfo, InitFragmentKey,
  InitFragmentStage, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact, RuntimeCondition,
  RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

use crate::import_dependency::module_id_rstest;

#[cacheable]
#[derive(Debug, Clone)]
pub struct MockModuleIdDependency {
  pub id: DependencyId,
  pub request: String,
  pub weak: bool,
  range: DependencyRange,
  optional: bool,
  factorize_info: FactorizeInfo,
  category: DependencyCategory,
  pub suffix: Option<String>,
  hoist: bool,
  await_factory: bool,
}

#[allow(clippy::too_many_arguments)]
impl MockModuleIdDependency {
  pub fn new(
    request: String,
    range: DependencyRange,
    weak: bool,
    optional: bool,
    category: DependencyCategory,
    suffix: Option<String>,
    hoist: bool,
    async_factory: bool,
  ) -> Self {
    Self {
      range,
      request,
      weak,
      optional,
      id: DependencyId::new(),
      factorize_info: Default::default(),
      category,
      suffix,
      hoist,
      await_factory: async_factory,
    }
  }
}

#[cacheable_dyn]
impl Dependency for MockModuleIdDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RstestMockModuleId
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
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
impl ModuleDependency for MockModuleIdDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn weak(&self) -> bool {
    self.weak
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for MockModuleIdDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(MockModuleIdDependencyTemplate::template_type())
  }
}

impl AsContextDependency for MockModuleIdDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct MockModuleIdDependencyTemplate;

impl MockModuleIdDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::RstestMockModuleId)
  }
}

impl DependencyTemplate for MockModuleIdDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext { init_fragments, .. } = code_generatable_context;

    let dep = dep
      .as_any()
      .downcast_ref::<MockModuleIdDependency>()
      .expect("MockModuleIdDependencyTemplate should only be used for MockModuleIdDependency");

    let module_id = module_id_rstest(
      code_generatable_context.compilation,
      &dep.id,
      &dep.request,
      dep.weak,
    );

    if dep.hoist && dep.await_factory {
      // Await exec init fragment.
      init_fragments.push(Box::new(ConditionalInitFragment::new(
        format!("await __webpack_require__.rstest_exec({module_id})\n"),
        InitFragmentStage::StageAsyncESMImports,
        i32::MAX - 1,
        InitFragmentKey::ESMImport(format!("{}_{}", module_id, "mock")),
        None,
        RuntimeCondition::Boolean(true),
      )));
    }

    source.replace(
      dep.range.start,
      dep.range.end,
      &format!("{}{}", module_id, dep.suffix.as_deref().unwrap_or("")),
      None,
    );
  }
}
